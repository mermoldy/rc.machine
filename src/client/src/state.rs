extern crate common;

use common::settings;
use common::types::MachineState;
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::time::Duration;

pub struct RemoteState {
    settings: settings::Settings,
    state: MachineState,
    stream: Option<std::net::TcpStream>,
    dirty: bool,
}

impl RemoteState {
    pub fn new(settings: settings::Settings) -> Result<Self, Box<dyn Error>> {
        Ok(RemoteState {
            settings: settings,
            stream: None,
            state: MachineState {
                backward: false,
                forward: false,
                left: false,
                right: false,
                lamp_enabled: false,
            },
            dirty: false,
        })
    }

    pub fn open(&mut self) {
        let addrs_iter = format!(
            "{}:{}",
            &self.settings.connection.host, &self.settings.connection.state_port
        )
        .to_socket_addrs()
        .unwrap();

        for addr in addrs_iter {
            info!("Connecting to {:?}...", addr);

            match TcpStream::connect_timeout(&addr, Duration::from_millis(1000)) {
                Ok(stream) => {
                    info!("Successfully connected to server in port 3333");
                    stream.set_nodelay(true).expect("set_nodelay call failed");
                    stream.set_ttl(5).expect("set_ttl call failed");
                    self.stream = Some(stream);
                }
                Err(e) => {
                    error!("Failed to connect: {}. Address: {}", e, addr);
                }
            }
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

    pub fn push(&mut self) -> Option<MachineState> {
        if self.dirty {
            self.push_state();
            self.dirty = false;
            Some(self.state)
        } else {
            None
        }
    }

    fn push_state(&mut self) {
        let bytes = bincode::serialize(&self.state).unwrap();
        match self.stream.as_ref() {
            Some(mut stream) => match stream.write(&bytes) {
                Ok(written) => {
                    debug!("Written {:?} bytes", written);

                    let mut data = [0 as u8; 1];
                    match stream.read_exact(&mut data) {
                        Ok(_) => {
                            debug!("Read {:?} bytes response", data.len());
                        }
                        Err(e) => {
                            error!(
                                "Failed to read the response: {}. Retrying push operation...",
                                e
                            );
                            self.push();
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to write: {:?}", e);
                    self.open();
                }
            },
            None => {
                error!("Failed to write. Connection is not initialized. Reconnecting...");
                self.open();
            }
        }
    }
}
