extern crate image;

use crate::settings;

use self::image::ImageFormat;

use std::error::Error;
use std::io::Read;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::time::Duration;

pub struct VideoStream {
    settings: settings::Settings,
}

pub struct VideoFrame {
    pub frame: image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>,
}

impl VideoStream {
    pub fn new(settings: settings::Settings) -> VideoStream {
        VideoStream { settings: settings }
    }

    pub fn connect(
        self,
        sender: std::sync::mpsc::Sender<VideoFrame>,
    ) -> Result<(), Box<dyn Error>> {
        let addrs_iter = format!(
            "{}:{}",
            &self.settings.connection.host, &self.settings.connection.video_port
        )
        .to_socket_addrs()?;
        for addr in addrs_iter {
            match TcpStream::connect_timeout(&addr, Duration::from_millis(5000)) {
                Ok(mut stream) => {
                    info!("Successfully connected to server port {:?}", addr);
                    let mut buffer: Vec<u8> = Vec::new();
                    let start_of_image: [u8; 2] = [255, 216];
                    let end_of_image: [u8; 2] = [255, 217];
                    loop {
                        let mut read_buffer = [0 as u8; 1024];

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
                                                let _ = sender.send(VideoFrame {
                                                    frame: img.rotate90().to_rgb(),
                                                });
                                            }
                                            Err(e) => error!("Failed to decode an image: {:?}", e),
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
                }
                Err(e) => {
                    warn!("Failed to connect to {}: {}", addr, e);
                }
            }
        }
        Ok(())
    }
}
