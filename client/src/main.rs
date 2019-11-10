#[macro_use]
extern crate log;

extern crate bincode;
extern crate config;
extern crate hidapi;
extern crate serde;
extern crate web_view;
use serde::{Deserialize, Serialize};
use std::thread;
use web_view::*;

use common::settings::Settings;

use std::io::{Read, Write};
use std::net::TcpStream;

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

#[derive(Serialize, Deserialize)]
pub struct KeyboardInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl PartialEq for KeyboardInput {
    fn eq(&self, other: &Self) -> bool {
        self.up == other.up
    }
}
impl Eq for KeyboardInput {}

pub struct App {
    pub settings: Settings,
    pub stream: Option<std::net::TcpStream>,
    pub last_key: Option<KeyboardInput>,
}

impl App {
    pub fn new() -> App {
        App {
            settings: Settings::new().unwrap(),
            stream: None,
            last_key: None,
        }
    }

    pub fn open(&mut self) {
        let url = format!("{}:{}", &self.settings.host, &self.settings.ctrl_port);
        match TcpStream::connect(url) {
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

    fn send(&mut self, input: KeyboardInput) {
        let bytes = bincode::serialize(&input).unwrap();
        match self.stream.as_ref() {
            Some(mut stream) => match stream.write(&bytes) {
                Ok(written) => {
                    debug!("Written {:?} bytes", written);
                }
                Err(e) => {
                    error!("Failed to write: {:?}", e);
                }
            },
            None => {
                error!("Failed to write. Connection is not intialized. Reconnecting...");
                self.open();
            }
        }
    }

    fn listen(&mut self) {
        let api = hidapi::HidApi::new().unwrap();
        let device = api.open(1356, 2508).unwrap();
        loop {
            let mut buf = [0u8; 10];
            let res = device.read(&mut buf[..]).unwrap();

            let button_input = &buf[..res][5];
            // https://www.psdevwiki.com/ps4/DS4-USB
            //
            let up = button_input & u8::pow(2, 5) != 0;
            let key = Some(KeyboardInput {
                up: up,
                down: false,
                left: false,
                right: false,
            });
            if key != self.last_key {
                self.last_key = key;
                self.send(KeyboardInput {
                    up: up,
                    down: false,
                    left: false,
                    right: false,
                });
            }
        }
    }
}

fn main() {
    env_logger::init();

    let main_app = thread::spawn(move || {
        let mut app = App::new();
        app.open();
        app.listen();
    });

    let settings = Settings::new().unwrap();
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

    main_app.join().unwrap();
}
