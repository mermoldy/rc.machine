pub mod gamepad;
pub mod state;
pub mod video;
pub mod window;

#[macro_use]
extern crate log;
extern crate common;
use common::settings;
use gamepad::Gamepad;
use state::RemoteState;
use video::{VideoFrame, VideoStream};
use window::Window;

use std::sync::mpsc;
use std::thread;

fn main() {
    println!("Initializing a logger...");
    env_logger::init();

    info!("Initializing a settings...");
    let settings = match settings::Settings::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize a settings: {:?}", e);
            std::process::exit(2);
        }
    };

    info!("Initializing a window...");
    let mut app = match Window::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize a window: {:?}", e);
            std::process::exit(3);
        }
    };

    info!("Initializing remote state connection...");
    let mut state = match RemoteState::new(settings.clone()) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize remote state: {:?}", e);
            std::process::exit(4);
        }
    };

    info!("Initializing a gamepad...");
    let mut gamepad = match Gamepad::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize a controller: {:?}", e);
            std::process::exit(5);
        }
    };

    info!("Initializing a video stream...");
    app.set_log("Initializing a video stream...");
    let video_stream = VideoStream::new(settings.clone());
    let (tx, rx): (
        std::sync::mpsc::Sender<VideoFrame>,
        std::sync::mpsc::Receiver<VideoFrame>,
    ) = mpsc::channel();
    thread::spawn(move || match video_stream.connect(tx) {
        Ok(_) => {
            info!("A video stream initialized");
        }
        Err(e) => {
            error!("Failed to initialize a video stream: {:?}", e);
            std::process::exit(3);
        }
    });

    loop {
        match rx.try_recv() {
            Ok(img) => {
                app.set_log("A video stream is established");
                app.set_image(img.data.rotate90());
            }
            _ => {}
        }

        match app.process_events() {
            window::Event::Exit => {
                info!("Exiting...");
                break;
            }
            window::Event::ButtonPressed(key) => match key {
                window::Key::L => state.enable_light(),
                window::Key::Up => state.forward(),
                window::Key::Down => state.backward(),
                window::Key::Right => state.right(),
                window::Key::Left => state.left(),
                _ => {}
            },
            window::Event::ButtonReleased(key) => match key {
                window::Key::L => state.disable_light(),
                window::Key::Up => state.stop(),
                window::Key::Down => state.stop(),
                window::Key::Right => state.straight(),
                window::Key::Left => state.straight(),
                _ => {}
            },
            _ => {}
        }

        match gamepad.process_events() {
            gamepad::Event::ButtonPressed(button) => match button {
                gamepad::Button::East => state.enable_light(),
                _ => {}
            },
            gamepad::Event::ButtonReleased(button) => match button {
                gamepad::Button::East => state.disable_light(),
                _ => {}
            },
            gamepad::Event::AxisChanged(axis, value) => match axis {
                gamepad::Axis::LeftStickX => {
                    if value > 0.5 {
                        state.right();
                    } else if value < -0.5 {
                        state.left();
                    } else {
                        state.straight();
                    }
                }
                _ => {}
            },
            gamepad::Event::ButtonChanged(button, value) => match button {
                gamepad::Button::RightTrigger2 => {
                    if value > 0.5 {
                        state.forward();
                    } else {
                        state.stop();
                    }
                }
                gamepad::Button::LeftTrigger2 => {
                    if value > 0.5 {
                        state.backward();
                    } else {
                        state.stop();
                    }
                }
                _ => {}
            },
            _ => {}
        }

        if state.push() {
            info!("Pushed the state.");
        }
    }
    info!("Terminated");
}
