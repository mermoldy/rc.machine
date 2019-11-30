#[macro_use]
extern crate log;

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use std::env;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};

use common::settings::Settings;

extern crate sysfs_gpio;

fn blink_my_led(led: u64, duration_ms: u64, period_ms: u64) -> sysfs_gpio::Result<()> {
    let my_led = Pin::new(led);
    my_led.with_exported(|| {
        my_led.set_direction(Direction::Low)?;
        let iterations = duration_ms / period_ms / 2;
        for _ in 0..iterations {
            my_led.set_value(0)?;
            sleep(Duration::from_millis(period_ms));
            my_led.set_value(1)?;
            sleep(Duration::from_millis(period_ms));
        }
        my_led.set_value(0)?;
        Ok(())
    })
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50];
    while match stream.read(&mut data) {
        Ok(size) => {
            info!("Readed {} bytes: {:?}", size, &data[0..size]);
            blink_my_led(18, 1000, 5).unwrap();
            // echo everything!
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
}

fn main() {
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
