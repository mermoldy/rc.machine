extern crate bincode;
use crate::utils;

use std::error;
use std::io;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

use crate::common::conn::MessageStream;
use crate::common::messages as msg;
use crate::common::settings;
use crate::common::types;
use simple_error::SimpleError as Error;
use std::collections::HashMap;
use std::time::Duration;

pub struct SessionPool {
    config: utils::Config,
    sessions: HashMap<String, Session>,
}

impl SessionPool {
    pub fn new(config: utils::Config) -> Self {
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

                    info!(
                        "Connected new client {}, performing handshake...",
                        stream.peer_addr()?
                    );

                    match stream.read_msg::<msg::RequestConnection>(&mut vec![]) {
                        Ok(message) => match message {
                            msg::RequestConnection {
                                session_id: None, ..
                            } => match message.conn_type {
                                msg::ConnectionType::Session(settings) => {
                                    self.open_session(stream, settings, message.token);
                                }
                                _ => {}
                            },
                            msg::RequestConnection {
                                session_id: Some(session_id),
                                ..
                            } => match self.lookup_session(&session_id) {
                                Some(session) => match message.conn_type {
                                    msg::ConnectionType::Video(settings) => {
                                        session.open_video_channel(settings);
                                    }
                                    msg::ConnectionType::Controller(settings) => {
                                        session.open_controller_channel(settings);
                                    }
                                    _ => {
                                        warn!("Unknown connection type");
                                    }
                                },
                                None => warn!("Session with ID {} not found", session_id),
                            },
                        },
                        Err(_) => {}
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
    ) -> Result<(), io::Error> {
        if !self.config.is_valid_token(token) {
            return match stream.write_msg(&msg::OpenSession {
                ok: false,
                session_id: None,
                error: Some("Invalid token".to_string()),
            }) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            };
        }

        let session_id = utils::gen_id(8);
        return match stream.write_msg(&msg::OpenSession {
            ok: true,
            session_id: Some(session_id.clone()),
            error: None,
        }) {
            Ok(_) => {
                let session = Session::new(session_id.clone(), stream);
                self.sessions.insert(session_id, session);
                Ok(())
            }
            Err(e) => Err(e),
        };
    }

    fn lookup_session(&mut self, session_id: &String) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    // fn handshake(&mut self, config: &utils::Config) -> Result<(), Box<dyn error::Error>> {
    //     debug!("Waiting for hello message...");
    //     match self.stream.read_msg(&mut vec![]) {
    //         Ok(message) => match message {
    //             msg::ClientHello { .. } => {
    //                 debug!("Received hello message");
    //                 let is_ok = config.is_valid_token(message.token);
    //                 if is_ok {}
    //                 let hello = msg::ServerHello {
    //                     ok: is_ok,
    //                     video_port: 0,
    //                     state_port: 0,
    //                 };
    //                 match self.stream.write_msg(&hello) {
    //                     Ok(_) => {
    //                         debug!("Sended hello message (ok: {})", hello.ok);
    //                         if hello.ok {
    //                             Ok(())
    //                         } else {
    //                             Err(Box::new(io::Error::new(
    //                                 io::ErrorKind::Other,
    //                                 format!("Client is rejected due to invalid token"),
    //                             )))
    //                         }
    //                     }
    //                     Err(e) => Err(Box::new(io::Error::new(
    //                         e.kind(),
    //                         format!("Failed to send hello message: {}", e),
    //                     ))),
    //                 }
    //             }
    //         },
    //         Err(e) => Err(Box::new(io::Error::new(
    //             e.kind(),
    //             format!("Failed to read hello message: {}", e),
    //         ))),
    //     }
    // }
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
        config: common::settings::Video,
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn open_controller_channel(
        &mut self,
        config: common::settings::Controller,
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }
}
