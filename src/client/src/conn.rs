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
use stoppable_thread as st_thread;

pub enum KeyEvents {
    Forward,
    Backward,
    Left,
    Right,
    LightOn,
    LightOff,
}

pub struct Session {
    main_conn: Option<TcpStream>,
    conn_timeout: u64,
    read_timeout: u64,
    settings: Settings,
    pub state: State,

    pub video_rx: Arc<Mutex<Option<mpsc::Receiver<types::VideoFrame>>>>,
    pub state_conn: Arc<Mutex<Option<TcpStream>>>,

    is_connected: sync::Arc<sync::atomic::AtomicBool>,
    threads: Vec<st_thread::StoppableHandle<()>>,
}

impl Session {
    pub fn new(settings: Settings) -> Self {
        Session {
            main_conn: None,
            state_conn: Arc::new(Mutex::new(None)),
            video_rx: Arc::new(Mutex::new(None)),
            conn_timeout: 300,
            read_timeout: 300,
            settings: settings,
            state: State::new(),
            is_connected: sync::Arc::new(sync::atomic::AtomicBool::default()),
            threads: vec![],
        }
    }

    pub fn connect(&mut self) -> Result<(), io::Error> {
        let addrs_str = format!(
            "{}:{}",
            &self.settings.connection.host, &self.settings.connection.port
        );
        let addrs_iter = addrs_str.to_socket_addrs()?;

        for addr in addrs_iter {
            info!("Connecting to {:?}...", addr);

            match self.open_session(&addr) {
                Ok((stream, session_id)) => {
                    info!("Opened session with ID {} on {}", session_id, addr);
                    self.main_conn = Some(stream);

                    info!("Connecting to the video stream...");
                    match self.open_video_connection(&addr, session_id) {
                        Ok(stream) => {
                            // self.video_conn = Some(stream);
                            self.stream_video(stream);

                            self.is_connected
                                .clone()
                                .store(false, sync::atomic::Ordering::Relaxed);
                            return Ok(());
                        }
                        Err(e) => {
                            error!("Unable to open video connection: {}", e);
                            return Err(e);
                        }
                    };

                    break;
                }
                Err(e) => {
                    info!("{:?}", e);
                    if e.kind() == io::ErrorKind::TimedOut
                        || e.kind() == io::ErrorKind::ConnectionRefused
                    {
                        warn!("Failed to connect to {}: {}", addr, e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        self.is_connected
            .clone()
            .store(false, sync::atomic::Ordering::Relaxed);

        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to connect to {}", addrs_str),
        ))
    }

    fn open_session(&self, addr: &std::net::SocketAddr) -> Result<(TcpStream, String), io::Error> {
        let mut stream =
            TcpStream::connect_timeout(&addr, Duration::from_millis(self.conn_timeout))?;

        stream.set_nodelay(true)?;
        stream.set_ttl(5)?;
        stream.set_read_timeout(Some(Duration::from_millis(self.read_timeout)))?;

        debug!("Sending open session message...");
        let open_session_msg = &msg::RequestConnection {
            token: self.settings.connection.token.clone(),
            session_id: None,
            conn_type: msg::ConnectionType::Session(self.settings.heartbeat.clone()),
        };
        stream.write_msg(open_session_msg)?;

        debug!("Sended open session message. Waiting for a response...");
        let open_session_resp = stream.read_msg::<msg::OpenSession>(&mut vec![])?;

        debug!("Received open session response.");
        if open_session_resp.ok {
            match open_session_resp.session_id {
                Some(session_id) => Ok((stream, session_id)),
                None => Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Response missing session ID",
                )),
            }
        } else {
            let err_msg = open_session_resp.error.unwrap_or("Unknown".to_string());
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Error when opening session: {}", err_msg),
            ))
        }
    }

    fn open_video_connection(
        &self,
        addr: &std::net::SocketAddr,
        session_id: String,
    ) -> Result<TcpStream, io::Error> {
        let mut stream =
            TcpStream::connect_timeout(&addr, Duration::from_millis(self.conn_timeout))?;

        stream.set_nodelay(true)?;
        stream.set_ttl(5)?;
        stream.set_read_timeout(Some(Duration::from_millis(self.read_timeout)))?;

        debug!("Sending open video message...");
        let open_session_msg = &msg::RequestConnection {
            token: self.settings.connection.token.clone(),
            session_id: Some(session_id),
            conn_type: msg::ConnectionType::Video(self.settings.video.clone()),
        };
        stream.write_msg(open_session_msg)?;

        debug!("Sended open video message. Waiting for a response...");
        let open_video_resp = stream.read_msg::<msg::OpenVideoConnection>(&mut vec![])?;
        if !open_video_resp.ok {
            let err_msg = open_video_resp.error.unwrap_or("Unknown".to_string());
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to open video stream. {}", err_msg),
            ))
        } else {
            debug!("Video connection is opened.");
            Ok(stream)
        }
    }

    pub fn set_connection(&mut self, rx: Option<mpsc::Receiver<types::VideoFrame>>) {
        let receiver = self.video_rx.clone();

        let mut value = receiver.try_lock().unwrap();
        *value = rx;
    }

    fn stream_video(&mut self, mut stream: TcpStream) {
        let (video_tx, video_rx): (
            mpsc::Sender<types::VideoFrame>,
            mpsc::Receiver<types::VideoFrame>,
        ) = mpsc::channel();

        self.set_connection(Some(video_rx));

        let mut video_thread = st_thread::spawn(move |stopped| {
            while !stopped.get() {
                match stream.read_msg::<msg::VideoFrame>(&mut vec![]) {
                    Ok(frame) => {
                        match image::load_from_memory_with_format(&frame.data, ImageFormat::Jpeg) {
                            Ok(img) => {
                                match video_tx.send(types::VideoFrame {
                                    image: img.rotate90().to_rgb(),
                                }) {
                                    Ok(_) => {
                                        //info!("Readed frame");
                                    }
                                    Err(e) => {
                                        warn!("{}", e);
                                    }
                                };
                            }
                            Err(e) => error!("Failed to decode an image: {:?}", e),
                        }
                    }
                    Err(e) => {
                        warn!("{}", e);
                    }
                }
                thread::sleep(Duration::from_millis(20));
            }
        });
        self.threads.insert(0, video_thread);
    }

    pub fn disconnect(&mut self) -> Result<(), io::Error> {
        while let Some(thread) = self.threads.pop() {
            let join_handle = thread.stop();
            let result = join_handle.join().unwrap();
        }

        self.is_connected
            .clone()
            .store(false, sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    pub fn try_push(&mut self) -> Option<MachineState> {
        info!("try_push");
        if self.state.dirty {
            self.push_state(&self.state.state);
            self.state.dirty = false;
            Some(self.state.state)
        } else {
            None
        }
    }

    fn push_state(&self, state: &types::MachineState) -> Result<(), io::Error> {
        // if !self.is_connected() {
        //     warn!("Cannot push data. Client is not connected.")
        // }
        info!("001");
        match self.state_conn.try_lock() {
            Ok(mut state_conn) => match *state_conn {
                Some(ref mut s) => {
                    info!("002");

                    s.write_msg(state);
                }
                None => {
                    info!("Not connected");
                }
            },
            Err(e) => {
                info!("Cannot unlock");
            }
        };

        Ok(())

        // match &self.state_conn {
        //     Some(conn) => {
        //         //
        //     }
        //     None => {}
        // }
    }

    pub fn send(&mut self, e: KeyEvents) {
        info!("Event");
    }

    pub fn receive(&mut self) -> types::VideoFrame {
        info!("Event");
        types::VideoFrame { image: vec![] }
    }
}

pub struct State {
    state: MachineState,
    dirty: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            state: MachineState {
                backward: false,
                forward: false,
                left: false,
                right: false,
                lamp_enabled: false,
            },
            dirty: false,
        }
    }

    pub fn forward(&mut self) {
        if !self.state.forward {
            self.state.forward = true;
            self.dirty = true;
        }
    }

    pub fn backward(&mut self) {
        if !self.state.backward {
            self.state.backward = true;
            self.dirty = true;
        }
    }

    pub fn stop(&mut self) {
        if self.state.forward || self.state.backward {
            self.state.forward = false;
            self.state.backward = false;
            self.dirty = true;
        }
    }

    pub fn left(&mut self) {
        if !self.state.left {
            self.state.left = true;
            self.dirty = true;
        }
    }

    pub fn right(&mut self) {
        if !self.state.right {
            self.state.right = true;
            self.dirty = true;
        }
    }

    pub fn straight(&mut self) {
        if self.state.left || self.state.right {
            self.state.left = false;
            self.state.right = false;
            self.dirty = true;
        }
    }

    pub fn enable_light(&mut self) {
        if !self.state.lamp_enabled {
            self.state.lamp_enabled = true;
            self.dirty = true;
        }
    }

    pub fn disable_light(&mut self) {
        if self.state.lamp_enabled {
            self.state.lamp_enabled = false;
            self.dirty = true;
        }
    }
}
