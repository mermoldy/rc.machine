extern crate web_view;
extern crate hidapi;
extern crate config;
extern crate serde;
extern crate bincode;
use std::thread;
use web_view::*;
use serde::{Serialize, Deserialize};

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
    pub last_key: Option<KeyboardInput>
}

impl App {
    pub fn new() -> App {
        let settings = Settings::new();
        println!("Config: {:?}", settings);
        App {
            settings: settings.unwrap(),
            stream: None,
            last_key: None,
        }
    }

    pub fn open(&mut self) {
        match TcpStream::connect(&self.settings.control.url) {
            Ok(stream) => {
                println!("Successfully connected to server in port 3333");
                self.stream = Some(stream);

                // let mut data = [0 as u8; 6]; // using 6 byte buffer
                // match stream.read_exact(&mut data) {
                //     Ok(_) => {
                //         if &data == msg {
                //             println!("Reply is ok!");
                //         } else {
                //             let text = from_utf8(&data).unwrap();
                //             println!("Unexpected reply: {}", text);
                //         }
                //     }
                //     Err(e) => {
                //         println!("Failed to receive data: {}", e);
                //     }
                // }
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
        println!("Terminated.");
    }

    fn send(&mut self, message: &[u8; 6]) {
        // let msg = b"Hello!";
        println!("Sent {:?}, awaiting reply...", message);
        self.stream.as_ref().unwrap().write(message).unwrap();
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
            let key = Some(KeyboardInput{up: up, down: false, left: false, right: false});
            if key != self.last_key {
                let bytes = bincode::serialize(&key).unwrap();
                self.last_key = key;
                println!("Up: {:?}",  bytes);
                println!("Updated!");
            }
        }
    }
}

fn main() {
    let main_app = thread::spawn(move || {
        let mut app = App::new();
        app.listen();
    });

    let settings = Settings::new().unwrap();
    web_view::builder()
        .title("Cat.Hunter")
        .content(Content::Url(settings.stream.url))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();

    main_app.join().unwrap();
}
