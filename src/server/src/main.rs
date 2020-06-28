pub mod camera;
pub mod conn;
pub mod machine;
pub mod utils;

#[macro_use]
extern crate log;
extern crate bincode;
extern crate common;
extern crate log4rs;
extern crate log_panics;
extern crate rand;
extern crate signal_hook;
extern crate simple_error;
extern crate sysfs_gpio;

use std::thread;
use std::time;

fn main() {
    println!("Starting...");
    let signals = signal_hook::iterator::Signals::new(&[signal_hook::SIGINT, signal_hook::SIGTERM])
        .expect("Unable to register signal handler.");
    match utils::init_logger() {
        Err(e) => {
            println!("Unable to initialize logger: {}", e);
            println!("Exiting...");
            std::process::exit(1);
        }
        _ => {}
    }

    info!("Initializing configuration...");
    let config = match utils::Config::from_env() {
        Ok(res) => res,
        Err(e) => {
            error!("{}", e);
            error!("Exiting...");
            std::process::exit(2);
        }
    };

    info!("Initializing GPIO...");
    let mut machine = machine::Machine::new();
    machine.export();

    info!("Initializing session pool on {} port...", config.port);
    let mut session_pool = conn::SessionPool::new(config);
    thread::spawn(move || match session_pool.listen() {
        Ok(_) => {}
        Err(e) => {
            error!("{}", e);
            error!("Exiting...");
            std::process::exit(3);
        }
    });

    info!("Starting event loop...");
    let recv_timeout = time::Duration::from_millis(100);
    loop {
        // match &session_pool.recv_state(recv_timeout) {
        //     Some(state) => {
        //         // info!("Got state: {:?}", state);
        //         machine.update(state);
        //     }
        //     _ => {}
        // }
        for sig in signals.pending() {
            info!("Received signal {:?}, exiting...", sig);
            machine.unexport();
            std::process::exit(sig);
        }
    }
}
