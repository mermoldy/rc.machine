extern crate bincode;

use simple_error::SimpleError as Error;
use std::collections::HashMap;
use std::error;
use std::io;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::camera;
use crate::common::conn::MessageStream;
use crate::common::messages as msg;
use crate::common::settings;
use crate::common::types;
use crate::utils;

pub struct SessionPool {
    config: utils::Config,
    sessions: HashMap<String, Session>,
}

impl SessionPool {
    pub fn new(config: utils::Config) -> Self {
        let (tx, rx): (
            std::sync::mpsc::Sender<types::MachineState>,
            std::sync::mpsc::Receiver<types::MachineState>,
        ) = mpsc::channel();

        SessionPool {
            config: config,
            sessions: HashMap::new(),
        }
    }

    pub fn listen(&mut self) -> Result<(), Box<dyn error::Error>> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", &self.config.port))?;
        listener.set_ttl(5)?;
        info!("Server listening on port {:?}", &self.config.port);
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    stream.set_nodelay(true)?;
                    stream.set_ttl(5)?;
                    stream.set_read_timeout(Some(Duration::from_millis(1000)))?;

                    info!("Connecting new client {}...", stream.peer_addr()?);
                    match stream.read_msg::<msg::RequestConnection>(&mut vec![]) {
                        Ok(message) => match message {
                            msg::RequestConnection {
                                session_id: None, ..
                            } => match message.conn_type {
                                msg::ConnectionType::Session(settings) => {
                                    match self.open_session(stream, settings, message.token) {
                                        Ok(session_id) => {
                                            error!("Opened a session with ID: {}", session_id);
                                        }
                                        Err(e) => error!("Failed to open a session: {}", e),
                                    };
                                }
                                _ => {
                                    error!("Unknown message type: {:?}", message.conn_type);
                                }
                            },
                            msg::RequestConnection {
                                session_id: Some(session_id),
                                ..
                            } => match self.lookup_session(&session_id) {
                                Some(session) => match message.conn_type {
                                    msg::ConnectionType::Video(settings) => {
                                        session.open_video_channel(stream, settings)?;
                                    }
                                    msg::ConnectionType::Controller(settings) => {
                                        session.open_controller_channel(stream, settings)?;
                                    }
                                    _ => {
                                        error!("Unknown message type: {:?}", message.conn_type);
                                    }
                                },
                                None => warn!("Session with ID {} not found", session_id),
                            },
                        },
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }
        drop(listener);

        Ok(())
    }

    fn is_valid_token(&mut self, token: String) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn open_session(
        &mut self,
        mut stream: TcpStream,
        config: common::settings::Heartbeat,
        token: String,
    ) -> Result<String, Error> {
        if !self.config.is_valid_token(token) {
            return match stream.write_msg(&msg::OpenSession {
                ok: false,
                session_id: None,
                error: Some("Invalid token".to_string()),
            }) {
                Ok(_) => Err(Error::new("Session is rejected due to invalid token.")),
                Err(e) => Err(Error::new(format!("{}", e))),
            };
        }

        let session_id = utils::gen_id(24);
        return match stream.write_msg(&msg::OpenSession {
            ok: true,
            session_id: Some(session_id.clone()),
            error: None,
        }) {
            Ok(_) => {
                let session = Session::new(session_id.clone(), stream);
                self.sessions.insert(session_id.clone(), session);
                Ok(session_id)
            }
            Err(e) => Err(Error::new(format!("{}", e))),
        };
    }

    fn lookup_session(&mut self, session_id: &String) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    pub fn recv_state(&self, timeout: std::time::Duration) -> Option<&types::MachineState> {
        None
    }
}

pub struct Session {
    pub id: String,
    conn: TcpStream,
    video_conn: Option<TcpStream>,
    state_conn: Option<TcpStream>,
}

impl Session {
    pub fn new(id: String, conn: TcpStream) -> Self {
        Session {
            id: id,
            conn: conn,
            video_conn: None,
            state_conn: None,
        }
    }

    fn open_video_channel(
        &mut self,
        mut stream: TcpStream,
        config: common::settings::Video,
    ) -> Result<(), Box<dyn error::Error>> {
        thread::spawn(move || {
            extern crate rscam;

            match rscam::new(config.device.as_str()) {
                Ok(mut camera) => {
                    match camera.start(&rscam::Config {
                        interval: (1, config.max_framerate as u32),
                        resolution: config.resolution,
                        format: b"MJPG",
                        nbuffers: 32,
                        field: rscam::FIELD_NONE,
                    }) {
                        Ok(_) => {
                            let _ = stream.write_msg(&msg::OpenVideoConnection {
                                ok: true,
                                error: None,
                            });
                            loop {
                                match camera.capture() {
                                    Ok(mut frame) => match stream.write_msg(&msg::VideoFrame {
                                        data: frame.to_vec(),
                                    }) {
                                        Err(e) => {
                                            error!(
                                                "Failed to send VideoFrame: {:?}. Stopping video stream...",
                                                e
                                            );
                                            break;
                                        }
                                        _ => {}
                                    },
                                    Err(e) => {
                                        error!("Unable to take picture: {:?}", e);
                                    }
                                }
                                std::thread::sleep_ms(5);
                            }
                        }
                        Err(e) => {
                            let _ = stream.write_msg(&msg::OpenVideoConnection {
                                ok: false,
                                error: Some(format!("Failed to start the stream: {}", e)),
                            });
                        }
                    }
                }
                Err(e) => {
                    let _ = stream.write_msg(&msg::OpenVideoConnection {
                        ok: false,
                        error: Some(format!("Failed to initialize video device: {}", e)),
                    });
                }
            }
        });

        Ok(())
    }

    fn open_controller_channel(
        &mut self,
        mut stream: TcpStream,
        config: common::settings::Controller,
    ) -> Result<(), Box<dyn error::Error>> {
        let open_ctrl_msg = stream.write_msg(&msg::OpenControllerConnection {
            ok: true,
            error: None,
        });

        thread::spawn(move || loop {
            let s = stream.read_msg::<types::MachineState>(&mut vec![]);
            debug!("State: {:?}", s);
        });

        Ok(())
    }
}
