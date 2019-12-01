#[macro_use]
extern crate log;
extern crate bincode;
extern crate config;
extern crate gfx_device_gl;
extern crate gfx_graphics;
extern crate gilrs;
extern crate hidapi;
extern crate piston;
extern crate piston_window;
extern crate serde;
extern crate web_view;

use common::settings::Settings;
use common::types::MachineState;
use gilrs::{Event, Gilrs};
use piston::event_loop::*;
use piston::input;
use piston::input::*;
use piston::window::WindowSettings;
use piston_window::PistonWindow;
use piston_window::Viewport;
use piston_window::*;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::io::{Read, Write};
use std::mem;
use std::net::TcpStream;
use std::path::Path;
use std::ptr;
use std::str::from_utf8;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use web_view::*;

extern crate texture;

use texture::{CreateTexture, Format};

// class Signals(int, enum.Enum):

//     # engine signals
//     move_forward = 1
//     move_backward = 2
//     move_left = 3
//     move_right = 4
//     stop_forward = 5
//     stop_backward = 6
//     stop_left = 7
//     stop_right = 8
//     stop = 20

//     # light signals
//     enable_light = 21
//     disable_light = 22
//     trigger_light = 23

// class SignalResult(int, enum.Enum):
//     ok = 0
//     error = 1

pub struct App {
    pub settings: Settings,
    pub stream: Option<std::net::TcpStream>,
    pub state: Option<MachineState>,
}
use std::net::ToSocketAddrs;
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

use gilrs::{Axis, Button, EventType};

fn main() {
    println!("Initializing a client...");
    env_logger::init();
    let settings = Settings::new().unwrap();

    let mut gilrs = Gilrs::new().unwrap();
    thread::spawn(move || {
        let mut app = App::new();
        app.open();

        info!("Starting event loop...");
        loop {
            while let Some(Event { id, event, time }) = gilrs.next_event() {
                let mut state = MachineState {
                    backward: false,
                    forward: false,
                    left: false,
                    right: false,
                    lamp_enabled: false,
                };
                //println!("New event from {}: {:?}", id, event);
                match event {
                    EventType::ButtonPressed(button, code) => match button {
                        Button::East => {
                            state.lamp_enabled = true;
                            println!("lamb enabled");
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
                            println!("X axis {:?}", value);
                            app.update(state);
                        }
                        _ => {}
                    },
                    EventType::ButtonChanged(button, value, code) => match button {
                        Button::RightTrigger2 => {
                            state.forward = (value == 1.0);
                            app.update(state);
                        }
                        Button::LeftTrigger2 => {
                            state.backward = (value == 1.0);
                            app.update(state);
                        }
                        _ => {}
                    },
                    EventType::ButtonReleased(button, code) => match button {
                        Button::East => {
                            state.lamp_enabled = false;
                            println!("lamb disabled");
                            app.update(state);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    });

    web_view::builder()
        .title("Cat.Hunter")
        .content(Content::Url(format!(
            "http://{}:{}",
            &settings.host, &settings.http_stream_port
        )))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn listen_stream(sender: std::sync::mpsc::Sender<Message>) {
    match TcpStream::connect("192.168.88.251:8081") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 8081");
            let mut buffer: Vec<u8> = Vec::new();
            let start_marker: [u8; 2] = [255, 216];
            let end_marker: [u8; 2] = [255, 217];
            let start = Instant::now();
            let mut i = 0;

            loop {
                let mut read_buffer = [0 as u8; 1024];

                match stream.read_exact(&mut read_buffer) {
                    Ok(_) => {
                        buffer.extend_from_slice(&read_buffer);

                        match find_subsequence(&buffer, &end_marker) {
                            Some(body) => match find_subsequence(&buffer, &start_marker) {
                                Some(header) => {
                                    let mut vec2 = buffer.split_off(body + 2);
                                    let mut data = buffer.split_off(header);
                                    buffer.clear();
                                    buffer.extend(vec2);
                                    i = i + 1;
                                    sender.send(Message {
                                        data: data.to_vec(),
                                    });
                                    println!(
                                        "Sended {:?}, Buffer: {:?}, Total frames: {:?}",
                                        data.len(),
                                        buffer.len(),
                                        i
                                    );

                                    if i % 10 == 0 {
                                        let duration = start.elapsed();
                                        let d = duration.as_secs();
                                        if d != 0 {
                                            println!(
                                            "Time elapsed in expensive_function() is: {:?}, FPS: {:?}, att: {:?}",
                                            duration,
                                            i / d,
                                            i,
                                        );
                                        }
                                    }
                                }
                                _ => {}
                            },
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

#[derive(Debug)]
struct Message {
    data: Vec<u8>,
}

// `Texture::from_image` leaks :(
fn main_piston() {
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [720, 576])
        .exit_on_esc(true)
        .build()
        .unwrap();
    println!("Press C to turn capture cursor on/off");
    let (tx, rx) = mpsc::channel();
    let th = thread::spawn(move || {
        listen_stream(tx);
    });

    let mut gilrs = Gilrs::new().unwrap();
    let mut events = Events::new(EventSettings::new().lazy(false));
    let ctx = &mut window.create_texture_context();
    let stx = &TextureSettings::new();
    let mut run = true;

    while run {
        match events.next(&mut window) {
            Some(e) => {
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
                    println!("Exited!!");
                    run = false;
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
            println!("{:?} New event from {}: {:?}", time, id, event);
        }

        match rx.try_recv() {
            Ok(received) => {
                let img = load_from_memory_with_format(&received.data, ImageFormat::JPEG)
                    .unwrap()
                    .to_rgba();
                match Texture::from_image(ctx, &img, stx) {
                    Ok(txt) => {
                        draw(&mut window, |c, g, _| {
                            clear([1.0; 4], g);
                            image(&txt, c.transform, g);
                        });
                        println!("Updated");
                    }
                    _ => println!("Failed !!!11"),
                }
            }
            Err(_) => {}
        }
    }

    th.join();
}
extern crate image;

use image::load_from_memory_with_format;
use image::{DynamicImage, ImageFormat, RgbaImage};

fn draw<F, U>(window: &mut PistonWindow, f: F) -> Option<U>
where
    F: FnOnce(Context, &mut G2d, &mut gfx_device_gl::Device) -> U,
{
    // window.window.make_current();
    let device = &mut window.device;
    let res = window.g2d.draw(
        &mut window.encoder,
        &window.output_color,
        &window.output_stencil,
        Viewport {
            draw_size: [640, 480],
            rect: [1000, 1000, 1000, 1000],
            window_size: [640.0, 480.0],
        },
        |c, g| f(c, g, device),
    );
    window.encoder.flush(device);
    None
}
