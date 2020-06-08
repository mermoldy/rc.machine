extern crate image;

use crate::settings;
use std::time::{Duration, Instant};

use self::image::ImageFormat;

use std::collections::VecDeque;
use std::error::Error;
use std::io;
use std::io::Read;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct VideoStream {
    settings: settings::Settings,
    is_connected: Arc<Mutex<bool>>,
    thread: Option<thread::JoinHandle<()>>,
}

pub struct VideoFrame {
    pub frame: image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>,
}

pub enum Events {
    Message(VideoFrame),
    Disconnect,
}

impl VideoStream {
    pub fn new(settings: settings::Settings) -> VideoStream {
        VideoStream {
            settings: settings,
            is_connected: Arc::new(Mutex::new(false)),
            thread: None,
        }
    }

    pub fn connect(
        &mut self,
        sender: std::sync::mpsc::Sender<Events>,
    ) -> Result<(), Box<dyn Error>> {
        let addr_str = format!(
            "{}:{}",
            &self.settings.connection.host, &self.settings.connection.video_port
        );
        let addrs_iter = addr_str.to_socket_addrs()?;
        for addr in addrs_iter {
            match TcpStream::connect_timeout(&addr, Duration::from_millis(5000)) {
                Ok(mut stream) => {
                    let tx = sender.clone();

                    let c1 = self.is_connected.clone();
                    let mut is_conn = c1.lock().unwrap();
                    *is_conn = true;

                    info!("Successfully connected to server port {:?}", addr);
                    let mut buffer: Vec<u8> = Vec::new();
                    let start_of_image: [u8; 2] = [255, 216];
                    let end_of_image: [u8; 2] = [255, 217];
                    let mut read_buffer = [0 as u8; 1024];
                    let c2 = self.is_connected.clone();

                    let th = thread::spawn(move || {
                        loop {
                            let is_conn2 = c2.lock().unwrap();
                            if !*is_conn2 {
                                break;
                            }
                            match stream.read_exact(&mut read_buffer) {
                                Ok(_) => {
                                    buffer.extend_from_slice(&read_buffer);
                                    match (
                                        twoway::find_bytes(&buffer, &start_of_image),
                                        twoway::find_bytes(&buffer, &end_of_image),
                                    ) {
                                        (Some(soi_marker), Some(eoi_marker)) => {
                                            let rest_buffer = buffer.split_off(eoi_marker + 2);
                                            let image_buffer = buffer.split_off(soi_marker);
                                            match image::load_from_memory_with_format(
                                                &image_buffer,
                                                ImageFormat::Jpeg,
                                            ) {
                                                Ok(img) => {
                                                    let frame = VideoFrame {
                                                        frame: img.rotate90().to_rgb(),
                                                    };
                                                    let _ = tx.send(Events::Message(frame));
                                                }
                                                Err(e) => {
                                                    error!("Failed to decode an image: {:?}", e)
                                                }
                                            }

                                            buffer.clear();
                                            buffer.extend(rest_buffer);
                                        }
                                        _ => {}
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to receive data: {}", e);
                                    std::thread::sleep(Duration::from_millis(100));
                                }
                            }
                        }
                        tx.send(Events::Disconnect);
                        info!("Disconnected");
                    });
                    self.thread = Some(th);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to connect to {}: {}", addr, e);
                }
            }
        }
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to connect to {}", addr_str),
        )))
    }

    pub fn disconnect(&self) {
        let c1 = self.is_connected.clone();
        let mut is_conn = c1.lock().unwrap();
        *is_conn = false;
        // Ok(())
    }
}

pub struct FPSCounter {
    frames: VecDeque<Instant>,
}

impl FPSCounter {
    pub fn new(limit: u8) -> FPSCounter {
        FPSCounter {
            frames: VecDeque::with_capacity(limit as usize),
        }
    }

    pub fn tick(&mut self) -> u8 {
        let now = Instant::now();
        let second_ago = now - Duration::from_secs(1);

        while self.frames.front().map_or(false, |t| *t < second_ago) {
            self.frames.pop_front();
        }

        self.frames.push_back(now);
        self.frames.len() as u8
    }
}
