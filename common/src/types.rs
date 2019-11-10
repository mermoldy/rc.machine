use config::{Config, ConfigError, File};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct KeyboardInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}
