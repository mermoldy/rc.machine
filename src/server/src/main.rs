pub mod conn;
pub mod machine;
pub mod utils;

#[macro_use]
extern crate log;
extern crate bincode;
extern crate chrono;
extern crate common;
extern crate log4rs;
extern crate log_panics;
extern crate rand;
extern crate signal_hook;
extern crate simple_error;
extern crate sysfs_gpio;

use std::sync;
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
    let machine_mutex = sync::Arc::new(sync::Mutex::new(machine));

    info!("Initializing session pool on {} port...", config.port);
    let mut session_pool = conn::SessionPool::new(config, machine_mutex.clone());

    thread::spawn(move || {
        match session_pool.listen() {
            Ok(_) => {
                info!("Exiting...");
                std::process::exit(1);
            }
            Err(e) => {
                error!("{}", e);
                error!("Exiting...");
                std::process::exit(3);
            }
        };
    });

    info!("Starting event loop...");
    loop {
        for sig in signals.pending() {
            info!("Received signal {:?}, exiting...", sig);
            let mut machine = machine_mutex.try_lock().expect("Failed to lock GPIO");
            machine.unexport();
            std::process::exit(sig);
        }
        thread::sleep(time::Duration::from_millis(200));
    }
}
