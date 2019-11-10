use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub host: String,
    pub ctrl_port: u16,
    pub http_stream_port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("Settings.toml"))?;
        s.try_into()
    }
}
