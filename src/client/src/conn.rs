extern crate bincode;

extern crate common;
extern crate gilrs;
extern crate image;

use crate::common::conn::MessageStream;
use crate::common::messages as msg;
use crate::common::types;

use crate::settings::Settings;
use std::time::Duration;

use self::image::ImageFormat;
use std::io;
use std::net::ToSocketAddrs;
use std::sync;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use std::net::TcpStream;
use stoppable_thread as st_thread;

pub struct Session {
    main_conn: Option<TcpStream>,
    conn_timeout: u64,
    read_timeout: u64,
    settings: Settings,

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
            conn_timeout: 1000,
            read_timeout: 1000,
            settings: settings,
            is_connected: sync::Arc::new(sync::atomic::AtomicBool::default()),
            threads: vec![],
        }
    }

    pub fn connect(
        &mut self,
    ) -> Result<
        (
            mpsc::Receiver<types::VideoFrame>,
            mpsc::Sender<types::MachineState>,
        ),
        io::Error,
    > {
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
                    let video_receiver = match self.open_video_connection(&addr, session_id.clone())
                    {
                        Ok(stream) => self.stream_video(stream),
                        Err(e) => {
                            error!("Unable to open video connection: {}", e);
                            return Err(e);
                        }
                    };

                    info!("Connecting to the control stream...");
                    let control_sender =
                        match self.open_control_connection(&addr, session_id.clone()) {
                            Ok(stream) => self.stream_control(stream),
                            Err(e) => {
                                error!("Unable to open controller connection: {}", e);
                                return Err(e);
                            }
                        };

                    self.is_connected
                        .clone()
                        .store(false, sync::atomic::Ordering::Relaxed);
                    return Ok((video_receiver, control_sender));
                }
                Err(e) => {
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
        if addr.is_ipv4() {
            stream.set_ttl(5)?;
        }
        stream.set_read_timeout(Some(Duration::from_millis(self.read_timeout)))?;

        debug!("Sending open session message...");
        let open_session_msg = &msg::RequestConnection {
            token: self.settings.connection.token.clone(),
            session_id: None,
            conn_type: msg::ConnectionType::Session(self.settings.heartbeat.clone()),
            // timestamp: std::time::Duration::new(),
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
        if addr.is_ipv4() {
            stream.set_ttl(5)?;
        }
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

    pub fn open_control_connection(
        &self,
        addr: &std::net::SocketAddr,
        session_id: String,
    ) -> Result<TcpStream, io::Error> {
        let mut stream =
            TcpStream::connect_timeout(&addr, Duration::from_millis(self.conn_timeout))?;

        stream.set_nodelay(true)?;
        if addr.is_ipv4() {
            stream.set_ttl(5)?;
        }
        stream.set_read_timeout(Some(Duration::from_millis(self.read_timeout)))?;

        debug!("Sending open control message...");
        let open_session_msg = &msg::RequestConnection {
            token: self.settings.connection.token.clone(),
            session_id: Some(session_id),
            conn_type: msg::ConnectionType::Controller(self.settings.controller.clone()),
        };
        stream.write_msg(open_session_msg)?;

        debug!("Sended open controller message. Waiting for a response...");
        let open_controller_resp = stream.read_msg::<msg::OpenControllerConnection>(&mut vec![])?;
        if !open_controller_resp.ok {
            let err_msg = open_controller_resp.error.unwrap_or("Unknown".to_string());
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to open controller stream. {}", err_msg),
            ))
        } else {
            debug!("Controller connection is opened.");
            Ok(stream)
        }
    }

    pub fn set_connection(&mut self, rx: Option<mpsc::Receiver<types::VideoFrame>>) {
        let receiver = self.video_rx.clone();

        let mut value = receiver.try_lock().unwrap();
        *value = rx;
    }

    fn stream_video(&mut self, mut stream: TcpStream) -> mpsc::Receiver<types::VideoFrame> {
        let (video_sender, video_receiver): (
            mpsc::Sender<types::VideoFrame>,
            mpsc::Receiver<types::VideoFrame>,
        ) = mpsc::channel();

        let video_thread = st_thread::spawn(move |stopped| {
            while !stopped.get() {
                match stream.read_msg::<msg::VideoFrame>(&mut vec![]) {
                    Ok(frame) => {
                        match image::load_from_memory_with_format(&frame.data, ImageFormat::Jpeg) {
                            Ok(img) => {
                                match video_sender.send(types::VideoFrame {
                                    image: img.rotate90().to_rgb(),
                                    timestamp_ms: frame.timestamp_ms,
                                }) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        warn!("Failed to process a frame: {:?}", e);
                                    }
                                };
                            }
                            Err(e) => warn!("Failed to decode a frame: {:?}", e),
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read a frame: {:?}", e);
                    }
                }
                thread::sleep(Duration::from_millis(15));
            }
        });
        self.threads.insert(0, video_thread);

        return video_receiver;
    }

    fn stream_control(&mut self, mut stream: TcpStream) -> mpsc::Sender<types::MachineState> {
        let (control_sender, control_receiver): (
            mpsc::Sender<types::MachineState>,
            mpsc::Receiver<types::MachineState>,
        ) = mpsc::channel();

        let controller_thread = st_thread::spawn(move |stopped| {
            while !stopped.get() {
                match control_receiver.try_recv() {
                    Ok(state) => match stream.write_msg(&state) {
                        Err(e) => {
                            warn!("Failed to write a state: {:?}", e);
                        }
                        _ => {}
                    },
                    Err(_) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        });
        self.threads.insert(0, controller_thread);

        return control_sender;
    }

    pub fn disconnect(&mut self) -> Result<(), io::Error> {
        while let Some(thread) = self.threads.pop() {
            let join_handle = thread.stop();
            join_handle.join().unwrap();
        }

        self.is_connected
            .clone()
            .store(false, sync::atomic::Ordering::Relaxed);

        Ok(())
    }
}
