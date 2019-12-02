#[macro_use]
extern crate log;
extern crate bincode;
extern crate log4rs;
extern crate log_panics;
extern crate signal_hook;
extern crate sysfs_gpio;
use common::settings::Settings;
use common::types::MachineState;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use signal_hook::{iterator::Signals, SIGINT, SIGTERM};
use std::env;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
use std::time;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};

struct Engine {
    pin_1: Pin,
    pin_2: Pin,
}

impl Engine {
    pub fn new(pin_1: u64, pin_2: u64) -> Engine {
        Engine {
            pin_1: Pin::new(pin_1),
            pin_2: Pin::new(pin_2),
        }
    }

    pub fn forward(&mut self) {
        self.pin_1.set_value(1).expect("Failed to set pin");
        self.pin_2.set_value(0).expect("Failed to set pin");
    }
    pub fn backward(&mut self) {
        self.pin_1.set_value(0).expect("Failed to set pin");
        self.pin_2.set_value(1).expect("Failed to set pin");
    }
    pub fn stop(&mut self) {
        self.pin_1.set_value(0).expect("Failed to set pin");
        self.pin_2.set_value(0).expect("Failed to set pin");
    }
    pub fn export(&mut self) {
        self.pin_1.export().expect("Failed to export pin");
        self.pin_2.export().expect("Failed to export pin");

        self.pin_1.set_direction(Direction::High).unwrap();
        self.pin_2.set_direction(Direction::High).unwrap();
    }
    pub fn unexport(&mut self) {
        self.pin_1.unexport().expect("Failed to unexport pin");
        self.pin_2.unexport().expect("Failed to unexport pin");
    }
}

struct Lamp {
    pin: Pin,
}

struct Machine {
    lamp: Lamp,
    right_engine: Engine,
    left_engine: Engine,
}

impl Lamp {
    pub fn new(pin: u64) -> Lamp {
        Lamp { pin: Pin::new(pin) }
    }

    pub fn export(&mut self) {
        self.pin.export().expect("Failed to export pin");
        self.pin.set_direction(Direction::High).unwrap();
    }
    pub fn unexport(&mut self) {
        self.pin.unexport().expect("Failed to unexport pin");
    }
    pub fn enable(&mut self) {
        self.pin.set_value(1);
    }
    pub fn disable(&mut self) {
        self.pin.set_value(0);
    }
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            lamp: Lamp::new(18),
            right_engine: Engine::new(17, 27),
            left_engine: Engine::new(23, 22),
        }
    }

    pub fn update(&mut self, state: MachineState) {
        if state.lamp_enabled {
            self.lamp.enable();
        } else {
            self.lamp.disable();
        }
        if state.forward {
            if state.left {
                self.right_engine.stop();
                self.left_engine.backward();
            } else if state.right {
                self.right_engine.backward();
                self.left_engine.stop();
            } else {
                self.right_engine.forward();
                self.left_engine.forward();
            }
        } else if state.backward {
            if state.left {
                self.right_engine.backward();
                self.left_engine.stop();
            } else if state.right {
                self.right_engine.stop();
                self.left_engine.backward();
            } else {
                self.right_engine.backward();
                self.left_engine.backward();
            }
        } else if state.left {
            self.right_engine.backward();
            self.left_engine.forward();
        } else if state.right {
            self.right_engine.forward();
            self.left_engine.backward();
        } else {
            self.right_engine.stop();
            self.left_engine.stop();
        }
    }
    pub fn export(&mut self) {
        self.lamp.export();
        self.right_engine.export();
        self.left_engine.export();

        self.lamp.disable();
        self.left_engine.stop();
        self.right_engine.stop();
    }
    pub fn unexport(&mut self) {
        self.lamp.unexport();
        self.right_engine.unexport();
        self.left_engine.unexport();
    }
}

fn main() {
    println!("Starting...");
    log_panics::init();

    let mut machine = Machine::new();
    machine.export();

    let signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S)} {l} {t}] {m}{n}",
        )))
        .build("/var/log/cat.hunter.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
    let (tx, rx): (
        std::sync::mpsc::Sender<MachineState>,
        std::sync::mpsc::Receiver<MachineState>,
    ) = mpsc::channel();

    thread::spawn(move || {
        listen_states(tx);
    });

    println!("Started");
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(state) => {
                info!("Got state: {:?}", state);
                machine.update(state);
            }
            Err(_) => {}
        }
        for sig in signals.pending() {
            info!("Received signal {:?}, exiting...", sig);
            machine.unexport();
            std::process::exit(sig);
        }
    }
}

fn listen_states(sender: std::sync::mpsc::Sender<MachineState>) {
    let settings = Settings::new().unwrap();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", &settings.ctrl_port)).unwrap();
    info!("Server listening on port {:?}", &settings.ctrl_port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("New connection: {}", stream.peer_addr().unwrap());
                handle_client(stream, &sender);
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }
    drop(listener);
}

fn handle_client(mut stream: TcpStream, sender: &std::sync::mpsc::Sender<MachineState>) {
    let addr = stream.peer_addr().unwrap();
    let mut data = [0 as u8; 50];

    loop {
        match stream.read(&mut data) {
            Ok(size) => {
                info!("Readed {} bytes, deserializing...", size);
                match bincode::deserialize::<MachineState>(&data[0..size]) {
                    Ok(state) => {
                        sender.send(state);
                    }
                    _ => {
                        error!("Failed to deserialize incoming data");
                    }
                };
                // stream.write(&data[0..size]).unwrap();
            }
            Err(_) => {
                error!("An error occurred, terminating connection with {}", addr);
                match stream.shutdown(Shutdown::Both) {
                    Ok(_) => {}
                    _ => error!("Failed to shutdown stream properly"),
                }
                break;
            }
        }
    }
}
