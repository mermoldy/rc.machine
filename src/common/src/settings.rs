extern crate config;
extern crate serde;

use self::config::{Config, ConfigError, File};
use self::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Connection {
    pub host: String,
    pub port: u16,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Video {
    pub device: String,
    pub resolution: (u32, u32),
    pub max_framerate: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Heartbeat {
    pub interval: u8,
    pub missed_beats: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Controller {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    pub connection: Connection,
    pub video: Video,
    pub heartbeat: Heartbeat,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("Settings.toml"))?;
        s.try_into()
    }
}
