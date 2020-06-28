extern crate bincode;
extern crate common;
extern crate log4rs;
extern crate log_panics;
extern crate rand;
extern crate signal_hook;
extern crate sysfs_gpio;

use common::settings;
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

fn stream_video(settings: settings::Settings) {
    let url = format!("0.0.0.0:{}", settings.connection.port);
    let listener = TcpListener::bind(&url).unwrap();
    info!("Listening started, ready to accept on {}...", url.as_str());

    for new_stream in listener.incoming() {
        match new_stream {
            Ok(mut stream) => match stream.peer_addr() {
                Ok(addr) => {
                    info!("Connected a new client {:?}...", addr);

                    let (tx, rx): (
                        std::sync::mpsc::Sender<std::vec::Vec<u8>>,
                        std::sync::mpsc::Receiver<std::vec::Vec<u8>>,
                    ) = mpsc::channel();

                    let video_settings = settings.video.clone();
                    let t = thread::spawn(move || {
                        listen_camera(tx, video_settings);
                    });

                    loop {
                        match rx.recv_timeout(Duration::from_millis(42)) {
                            Ok(mut frame) => match stream.write(&frame) {
                                Ok(size) => {
                                    //debug!("Written {:?} bytes", size);
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to write: {:?}. Closing connection {:?}.",
                                        e, addr
                                    );
                                    break;
                                }
                            },
                            Err(e) => {}
                        }
                    }
                    info!("Stopping camera stream...");
                    drop(rx);
                    t.join();
                    info!("Stopped camera stream");
                }
                Err(e) => {}
            },
            Err(e) => {
                error!("Cannot connect a new client: {:?}", e);
            }
        }
    }
}

fn listen_camera(
    sender: std::sync::mpsc::Sender<std::vec::Vec<u8>>,
    video_settings: settings::Video,
) -> rscam::Result<()> {
    #[cfg(target_arch = "arm")]
    {
        extern crate rscam;

        let mut camera = rscam::new(video_settings.device.as_str())?;
        camera.start(&rscam::Config {
            interval: (1, video_settings.max_framerate as u32),
            resolution: video_settings.resolution,
            format: b"MJPG",
            nbuffers: video_settings.max_framerate as u32,
            field: rscam::FIELD_NONE,
        })?;

        loop {
            match camera.capture() {
                Ok(mut frame) => match sender.send(frame.to_vec()) {
                    Err(e) => {
                        error!("Failed to send a frame: {:?}. Exiting...", e);
                        break;
                    }
                    _ => {}
                },
                Err(e) => {
                    error!("Unable to take picture: {:?}", e);
                }
            }
            std::thread::sleep_ms(42);
        }
    }
    Ok(())
}

// let video_settings = settings.clone();
// thread::spawn(move || {
//     stream_video(video_settings);
// });
