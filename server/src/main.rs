#[macro_use]
extern crate log;
extern crate bincode;
extern crate sysfs_gpio;

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use common::types::MachineState;
use std::env;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};
use common::settings::Settings;

struct Engine {
    rotation_pin: Pin,
    pwm_pin: Pin,
}

impl Engine {
    pub fn new(rotation_pin: u64, pwm_pin: u64) -> Engine {
        Engine {
            rotation_pin: Pin::new(rotation_pin),
            pwm_pin: Pin::new(pwm_pin),
        }
    }

    pub fn forward(&mut self) {
        self.rotation_pin.set_value(1).unwrap();
        self.pwm_pin.set_value(0).unwrap();
    }
    pub fn backward(&mut self) {
        self.rotation_pin.set_value(0).unwrap();
        self.pwm_pin.set_value(1).unwrap();
    }
    pub fn stop(&mut self) {
        self.rotation_pin.set_value(0).unwrap();
        self.pwm_pin.set_value(0).unwrap();
    }
    pub fn export(&mut self) {
        self.rotation_pin.export().unwrap();
        self.pwm_pin.export().unwrap();

        self.pwm_pin.set_direction(Direction::High).unwrap();
        self.rotation_pin.set_direction(Direction::High).unwrap();
    }
    pub fn unexport(&mut self) {
        self.rotation_pin.unexport().unwrap();
        self.pwm_pin.unexport().unwrap();
    }
}


fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50];

    let my_led = Pin::new(18);

    let mut right_engine = Engine::new(17, 27);
    let mut left_engine = Engine::new(23, 22);

    my_led.export().unwrap();
    right_engine.export();
    left_engine.export();
    my_led.set_direction(Direction::High).unwrap();

    while match stream.read(&mut data) {
        Ok(size) => {
            let buf = &data[0..size];
            let state: MachineState = bincode::deserialize(&buf).unwrap();
            info!("Readed {} state: {:?}", size, state);
            if state.lamp_enabled {
                my_led.set_value(1).unwrap();
            } else {
                my_led.set_value(0).unwrap();
            }

            if state.forward {
                if state.left {
                    right_engine.stop();
                    left_engine.backward();
                } else if state.right {
                    right_engine.backward();
                    left_engine.stop();
                } else {
                    right_engine.forward();
                    left_engine.forward();
                }
            } else if state.backward {
                if state.left {
                    right_engine.backward();
                    left_engine.stop();
                } else if state.right {
                    right_engine.stop();
                    left_engine.backward();
                } else {
                    right_engine.backward();
                    left_engine.backward();
                }
            } else if state.left {
                right_engine.backward();
                left_engine.forward();
            } else if state.right {
                right_engine.forward();
                left_engine.backward();
            } else {
                right_engine.stop();
                left_engine.stop();
            }
            // stream.write(&data[0..size]).unwrap();
            true
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
    my_led.unexport().unwrap();
    right_engine.unexport();
    left_engine.unexport();
}

fn main() {
    println!("Initializing a server...");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let settings = Settings::new().unwrap();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", &settings.ctrl_port)).unwrap();
    info!("Server listening on port {:?}", &settings.ctrl_port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }
    drop(listener);
}
