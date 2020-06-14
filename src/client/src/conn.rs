extern crate bincode;

extern crate common;
extern crate gilrs;
extern crate image;

use crate::common::conn::MessageStream;
use crate::common::messages as msg;
use crate::common::settings;
use crate::common::types;

use crate::settings::Settings;
use std::time::Duration;

use self::image::ImageFormat;
use std::error;
use std::io;
use std::net::ToSocketAddrs;
use std::sync;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use common::types::MachineState;
use simple_error::SimpleError as Error;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};

pub struct Session {
    // video_conn: VideoConnection,
    // state_conn: StateConnection,
    settings: Settings,
    is_connected: sync::Arc<sync::atomic::AtomicBool>,
}

impl Session {
    pub fn new(settings: Settings) -> Self {
        Session {
            // video_conn: video_conn,
            // state_conn: state_conn,
            settings: settings,
            is_connected: sync::Arc::new(sync::atomic::AtomicBool::default()),
        }
    }

    pub fn connect(&mut self) -> Result<(), Box<dyn error::Error>> {
        let addrs_iter = format!(
            "{}:{}",
            &self.settings.connection.host, &self.settings.connection.port
        )
        .to_socket_addrs()
        .unwrap();

        for addr in addrs_iter {
            info!("Connecting to {:?}...", addr);

            match TcpStream::connect_timeout(&addr, Duration::from_millis(1000)) {
                Ok(mut stream) => {
                    stream.set_nodelay(true).expect("set_nodelay call failed");
                    stream.set_ttl(5).expect("set_ttl call failed");
                    stream.set_read_timeout(Some(Duration::from_millis(1000)))?;

                    // stream.write(buf: &[u8])

                    info!("Successfully connected to server, performing handshake...");
                    self.open_session(&mut stream);
                    break;
                }
                Err(e) => {
                    warn!("Failed to connect: {}. Address: {}", e, addr);
                }
            }
        }
        self.is_connected
            .clone()
            .store(true, sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    fn open_session(&mut self, stream: &mut TcpStream) -> Result<msg::OpenSession, Error> {
        let hello = msg::RequestConnection {
            token: self.settings.connection.token.clone(),
            session_id: None,
            conn_type: msg::ConnectionType::Session(self.settings.heartbeat.clone()),
        };

        debug!("Sending hello message...");
        match stream.write_msg(&hello) {
            Ok(_) => {
                debug!("Sended hello message. Waiting for a response...");
                match stream.read_msg::<msg::OpenSession>(&mut vec![]) {
                    Ok(server_hello) => {
                        debug!("Received hello message.");
                        if server_hello.ok {
                            info!("Session is established.");
                            Ok(server_hello)
                        } else {
                            Err(Error::new("Session is rejected by the server"))
                        }
                    }
                    Err(e) => Err(Error::new(format!("{}", e))),
                }
            }
            Err(e) => Err(Error::new(format!("{}", e))),
        }
    }

    // pub fn disconnect(&mut self) {
    //     self.video_conn.disconnect();
    //     self.state_conn.disconnect();

    //     self.is_connected
    //         .clone()
    //         .store(false, sync::atomic::Ordering::Relaxed);
    // }

    // pub fn on_connected(&self) {
    //     self.process_video_events();
    //     self.process_gamepad_events();
    // }
}

// pub struct StateConnection {
//     settings: settings::Settings,
//     state: MachineState,
//     stream: Option<std::net::TcpStream>,
//     dirty: bool,
// }

// impl StateConnection {
//     pub fn new(settings: settings::Settings) -> Result<Self, Box<dyn Error>> {
//         Ok(StateConnection {
//             settings: settings,
//             stream: None,
//             state: MachineState {
//                 backward: false,
//                 forward: false,
//                 left: false,
//                 right: false,
//                 lamp_enabled: false,
//             },
//             dirty: false,
//         })
//     }

//     pub fn connect(&mut self) {
//         let addrs_iter = format!(
//             "{}:{}",
//             &self.settings.connection.host, &self.settings.connection.state_port
//         )
//         .to_socket_addrs()
//         .unwrap();

//         for addr in addrs_iter {
//             info!("Connecting to {:?}...", addr);

//             match TcpStream::connect_timeout(&addr, Duration::from_millis(1000)) {
//                 Ok(stream) => {
//                     info!("Successfully connected to server in port 3333");
//                     stream.set_nodelay(true).expect("set_nodelay call failed");
//                     stream.set_ttl(5).expect("set_ttl call failed");
//                     self.stream = Some(stream);
//                     break;
//                 }
//                 Err(e) => {
//                     error!("Failed to connect: {}. Address: {}", e, addr);
//                 }
//             }
//         }
//     }

//     pub fn is_connected(&mut self) -> bool {
//         self.stream.is_some()
//     }

//     pub fn disconnect(&mut self) {
//         match &self.stream {
//             Some(stream) => {
//                 stream
//                     .shutdown(Shutdown::Both)
//                     .expect("Disconnect call failed");
//                 self.stream = None;
//             }
//             None => {}
//         };
//     }

//     pub fn forward(&mut self) {
//         if !self.state.forward {
//             self.state.forward = true;
//             self.dirty = true;
//         }
//     }

//     pub fn backward(&mut self) {
//         if !self.state.backward {
//             self.state.backward = true;
//             self.dirty = true;
//         }
//     }

//     pub fn stop(&mut self) {
//         if self.state.forward || self.state.backward {
//             self.state.forward = false;
//             self.state.backward = false;
//             self.dirty = true;
//         }
//     }

//     pub fn left(&mut self) {
//         if !self.state.left {
//             self.state.left = true;
//             self.dirty = true;
//         }
//     }

//     pub fn right(&mut self) {
//         if !self.state.right {
//             self.state.right = true;
//             self.dirty = true;
//         }
//     }

//     pub fn straight(&mut self) {
//         if self.state.left || self.state.right {
//             self.state.left = false;
//             self.state.right = false;
//             self.dirty = true;
//         }
//     }

//     pub fn enable_light(&mut self) {
//         if !self.state.lamp_enabled {
//             self.state.lamp_enabled = true;
//             self.dirty = true;
//         }
//     }

//     pub fn disable_light(&mut self) {
//         if self.state.lamp_enabled {
//             self.state.lamp_enabled = false;
//             self.dirty = true;
//         }
//     }

//     pub fn push(&mut self) -> Option<MachineState> {
//         if self.dirty {
//             self.push_state();
//             self.dirty = false;
//             Some(self.state)
//         } else {
//             None
//         }
//     }

//     fn push_state(&mut self) {
//         if !self.is_connected() {
//             warn!("Cannot push data. Client is not connected.")
//         }
//         let bytes = bincode::serialize(&self.state).unwrap();
//         match self.stream.as_ref() {
//             Some(mut stream) => match stream.write(&bytes) {
//                 Ok(written) => {
//                     debug!("Written {:?} bytes", written);

//                     let mut data = [0 as u8; 1];
//                     match stream.read_exact(&mut data) {
//                         Ok(_) => {
//                             debug!("Read {:?} bytes response", data.len());
//                         }
//                         Err(e) => {
//                             error!(
//                                 "Failed to read the response: {}. Retrying push operation...",
//                                 e
//                             );
//                             self.push();
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     error!("Failed to write: {:?}", e);
//                     self.connect();
//                     self.push_state();
//                 }
//             },
//             None => {
//                 error!("Failed to write. Connection is not initialized. Reconnecting...");
//                 self.connect();
//             }
//         }
//     }
// }

// pub struct VideoConnection {
//     settings: settings::Settings,
//     receiver: Arc<Mutex<Option<mpsc::Receiver<VideoFrame>>>>,
// }

// pub struct VideoFrame {
//     pub frame: image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>,
// }

// impl VideoConnection {
//     pub fn new(settings: settings::Settings) -> VideoConnection {
//         VideoConnection {
//             settings: settings,
//             receiver: Arc::new(Mutex::new(None)),
//         }
//     }

//     pub fn connect(&mut self) -> Result<(), Box<dyn Error>> {
//         if self.is_connected() {
//             return Err(Box::new(io::Error::new(
//                 io::ErrorKind::Other,
//                 format!("Already connected"),
//             )));
//         }

//         let addr_str = format!(
//             "{}:{}",
//             &self.settings.connection.host, &self.settings.connection.video_port
//         );
//         let addrs_iter = addr_str.to_socket_addrs()?;
//         for addr in addrs_iter {
//             match TcpStream::connect_timeout(&addr, Duration::from_millis(5000)) {
//                 Ok(mut stream) => {
//                     let receiver = self.receiver.clone();
//                     let (tx, rx): (mpsc::Sender<VideoFrame>, mpsc::Receiver<VideoFrame>) =
//                         mpsc::channel();

//                     info!("Successfully connected to server port {:?}", addr);
//                     let mut buffer: Vec<u8> = Vec::new();
//                     let start_of_image: [u8; 2] = [255, 216];
//                     let end_of_image: [u8; 2] = [255, 217];
//                     let mut read_buffer = [0 as u8; 1024];

//                     thread::spawn(move || {
//                         loop {
//                             match receiver.try_lock() {
//                                 Ok(receiver) => {
//                                     if !receiver.is_some() {
//                                         break;
//                                     }
//                                 }
//                                 Err(_) => {}
//                             }

//                             match stream.read_exact(&mut read_buffer) {
//                                 Ok(_) => {
//                                     buffer.extend_from_slice(&read_buffer);
//                                     match (
//                                         twoway::find_bytes(&buffer, &start_of_image),
//                                         twoway::find_bytes(&buffer, &end_of_image),
//                                     ) {
//                                         (Some(soi_marker), Some(eoi_marker)) => {
//                                             let rest_buffer = buffer.split_off(eoi_marker + 2);
//                                             let image_buffer = buffer.split_off(soi_marker);
//                                             match image::load_from_memory_with_format(
//                                                 &image_buffer,
//                                                 ImageFormat::Jpeg,
//                                             ) {
//                                                 Ok(img) => {
//                                                     let frame = VideoFrame {
//                                                         frame: img.rotate90().to_rgb(),
//                                                     };
//                                                     let _ = tx.send(frame);
//                                                 }
//                                                 Err(e) => {
//                                                     error!("Failed to decode an image: {:?}", e)
//                                                 }
//                                             }

//                                             buffer.clear();
//                                             buffer.extend(rest_buffer);
//                                         }
//                                         _ => {}
//                                     }
//                                 }
//                                 Err(e) => {
//                                     error!("Failed to receive data: {}", e);
//                                     std::thread::sleep(Duration::from_millis(100));
//                                 }
//                             }
//                         }
//                         info!("Disconnected");
//                     });
//                     self.set_connection(Some(rx));
//                     return Ok(());
//                 }
//                 Err(e) => {
//                     warn!("Failed to connect to {}: {}", addr, e);
//                 }
//             }
//         }
//         Err(Box::new(io::Error::new(
//             io::ErrorKind::Other,
//             format!("Failed to connect to {}", addr_str),
//         )))
//     }

//     pub fn is_connected(&mut self) -> bool {
//         match self.receiver.clone().try_lock() {
//             Ok(receiver) => receiver.is_some(),
//             Err(_) => false,
//         }
//     }

//     pub fn set_connection(&mut self, rx: Option<mpsc::Receiver<VideoFrame>>) {
//         let receiver = self.receiver.clone();
//         let mut value = receiver.lock().unwrap();
//         *value = rx;
//     }

//     pub fn connection(&self) -> Arc<Mutex<Option<mpsc::Receiver<VideoFrame>>>> {
//         self.receiver.clone()
//     }

//     pub fn disconnect(&mut self) {
//         self.set_connection(None);
//     }
// }
