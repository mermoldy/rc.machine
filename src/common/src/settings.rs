extern crate config;
extern crate serde;

use self::config::{Config, ConfigError, File};
use self::serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Connection {
    pub host: String,
    pub state_port: u16,
    pub video_port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Video {
    pub device: String,
    pub resolution: (u32, u32),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub connection: Connection,
    pub video: Video,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("Settings.toml"))?;
        s.try_into()
    }
}
