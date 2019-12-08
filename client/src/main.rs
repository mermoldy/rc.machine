#[macro_use]
extern crate log;
extern crate bincode;
extern crate config;
extern crate gilrs;
extern crate hidapi;
extern crate image;
extern crate piston;
extern crate piston_window;
extern crate serde;
extern crate twoway;

use common::settings::Settings;
use common::types::MachineState;
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use image::ImageFormat;
use piston::event_loop::*;
use piston::input;
use piston::input::*;
use piston_window::*;
use piston_window::{EventSettings, Events, PistonWindow, Texture, WindowSettings};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use texture::TextureSettings;

struct Message {
    data: image::DynamicImage,
    index: u32,
}

pub struct App {
    pub settings: Settings,
    pub stream: Option<std::net::TcpStream>,
    pub state: Option<MachineState>,
}

impl App {
    pub fn new() -> App {
        App {
            settings: Settings::new().unwrap(),
            stream: None,
            state: None,
        }
    }

    pub fn open(&mut self) {
        let url = format!("{}:{}", &self.settings.host, &self.settings.ctrl_port);
        info!("Connecting to {:?}...", url);
        let addr: std::net::SocketAddr = url.parse().expect("Unable to parse socket address");
        match TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
            Ok(stream) => {
                info!("Successfully connected to server in port 3333");
                self.stream = Some(stream);
            }
            Err(e) => {
                error!("Failed to connect: {}", e);
            }
        }
    }

    fn read(&mut self) {
        match self.stream.as_ref() {
            Some(mut stream) => {
                let mut data = [0 as u8; 6]; // using 6 byte buffer
                match stream.read_exact(&mut data) {
                    Ok(_) => {
                        debug!("Readed {:?}", data);
                    }
                    Err(e) => {
                        error!("Failed to read: {:?}", e);
                    }
                }
            }
            None => {
                error!("Failed to read. Connection is not intialized. Reconnecting...");
                self.open();
            }
        }
    }
    fn update(&mut self, input: MachineState) {
        let s = Some(input);
        if self.state != s {
            self.state = s;
            self.push();
        } else {
            debug!("No chanhes in state");
        }
    }

    fn push(&mut self) {
        let bytes = bincode::serialize(&self.state.unwrap()).unwrap();
        match self.stream.as_ref() {
            Some(mut stream) => match stream.write(&bytes) {
                Ok(written) => {
                    debug!("Written {:?} bytes", written);
                }
                Err(e) => {
                    error!("Failed to write: {:?}", e);
                    self.open();
                }
            },
            None => {
                error!("Failed to write. Connection is not intialized. Reconnecting...");
                self.open();
            }
        }
    }
}

fn listen_stream(sender: std::sync::mpsc::Sender<Message>) {
    match TcpStream::connect("192.168.88.241:8081") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 8081");
            let mut buffer: Vec<u8> = Vec::new();
            let start_of_image: [u8; 2] = [255, 216];
            let end_of_image: [u8; 2] = [255, 217];
            loop {
                let mut read_buffer = [0 as u8; 1024];

                match stream.read_exact(&mut read_buffer) {
                    Ok(_) => {
                        buffer.extend_from_slice(&read_buffer);
                        match (
                            twoway::find_bytes(&buffer, &start_of_image),
                            twoway::find_bytes(&buffer, &end_of_image),
                        ) {
                            (Some(soi_marker), Some(eoi_marker)) => {
                                let rest_buffer = buffer.split_off(eoi_marker + 2);
                                let image_buffer = buffer.split_off(soi_marker);
                                let img = image::load_from_memory_with_format(
                                    &image_buffer,
                                    ImageFormat::JPEG,
                                )
                                .unwrap();
                                sender
                                    .send(Message {
                                        data: img,
                                        index: 0,
                                    })
                                    .unwrap();
                                buffer.clear();
                                buffer.extend(rest_buffer);
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }

    println!("Terminated.");
}

fn main() {
    println!("Initializing a client...");
    env_logger::init();

    let width = 800.0;
    let height = 600.0;
    let settings = Settings::new().unwrap();

    let mut window: PistonWindow = WindowSettings::new("Cat Hunter", [width, height])
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();
    println!("Press C to turn capture cursor on/off");
    let (tx, rx): (
        std::sync::mpsc::Sender<Message>,
        std::sync::mpsc::Receiver<Message>,
    ) = mpsc::channel();
    thread::spawn(move || {
        listen_stream(tx);
    });

    let mut gilrs = Gilrs::new().unwrap();
    let mut events = Events::new(EventSettings::new().lazy(false));
    let mut canvas = image::ImageBuffer::new(width as u32, height as u32);
    let mut texture_context = piston_window::TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let mut texture: piston_window::G2dTexture =
        Texture::from_image(&mut texture_context, &canvas, &TextureSettings::new()).unwrap();

    let mut app = App::new();
    app.open();

    loop {
        match events.next(&mut window) {
            Some(e) => {
                if let Some(_) = e.render_args() {
                    let ww = window.size().width;
                    let wh = window.size().height;
                    texture.update(&mut texture_context, &canvas).unwrap();
                    window.draw_2d(&e, |c, g, device| {
                        texture_context.encoder.flush(device);
                        piston_window::clear([1.0; 4], g);
                        piston_window::image(
                            &texture,
                            c.transform
                                .trans(0.0, 0.0)
                                .scale(1.0 * (ww / width), 1.0 * (wh / height)),
                            g,
                        );
                    });
                }

                if let Some(input::Button::Keyboard(key)) = e.press_args() {
                    if key == input::Key::C {
                        println!("Turned capture cursor on");
                    }

                    println!("Pressed keyboard key '{:?}'", key);
                }
                if let Some(args) = e.button_args() {
                    println!("Scancode {:?}", args.scancode);
                }
                if let Some(_) = e.close_args() {
                    println!("Exited!");
                    break;
                }
                if let Some(button) = e.release_args() {
                    match button {
                        input::Button::Keyboard(key) => {
                            println!("Released keyboard key '{:?}'", key)
                        }
                        input::Button::Mouse(button) => {
                            println!("Released mouse button '{:?}'", button)
                        }
                        input::Button::Controller(button) => {
                            println!("Released controller button '{:?}'", button)
                        }
                        input::Button::Hat(hat) => println!("Released controller hat `{:?}`", hat),
                    }
                }
            }
            None => {}
        }

        while let Some(Event { id, event, time }) = gilrs.next_event() {
            let mut state = MachineState {
                backward: false,
                forward: false,
                left: false,
                right: false,
                lamp_enabled: false,
            };
            match event {
                EventType::ButtonPressed(button, code) => match button {
                    Button::East => {
                        state.lamp_enabled = true;
                        app.update(state);
                    }
                    _ => {}
                },
                EventType::AxisChanged(button, value, code) => match button {
                    Axis::LeftStickX => {
                        if value > 0.9 {
                            state.right = true;
                            state.left = false;
                        } else if value < -0.9 {
                            state.right = false;
                            state.left = true;
                        } else {
                            state.right = false;
                            state.left = false;
                        }
                        app.update(state);
                    }
                    _ => {}
                },
                EventType::ButtonChanged(button, value, code) => match button {
                    Button::RightTrigger2 => {
                        state.forward = value == 1.0;
                        app.update(state);
                    }
                    Button::LeftTrigger2 => {
                        state.backward = value == 1.0;
                        app.update(state);
                    }
                    _ => {}
                },
                EventType::ButtonReleased(button, code) => match button {
                    Button::East => {
                        state.lamp_enabled = false;
                        app.update(state);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        match rx.try_recv() {
            Ok(img) => {
                canvas = img.data.rotate90().to_rgba();
            }
            _ => {}
        }
    }
}
