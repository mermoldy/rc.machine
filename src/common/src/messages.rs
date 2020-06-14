extern crate serde;

use self::serde::{Deserialize, Serialize};
use settings::{Controller, Heartbeat, Video};

#[derive(Serialize, Deserialize, Clone)]
pub enum ConnectionType {
    Session(Heartbeat),
    Video(Video),
    Controller(Controller),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RequestConnection {
    pub token: String,
    pub session_id: Option<String>,
    pub conn_type: ConnectionType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenSession {
    pub ok: bool,
    pub session_id: String,
    pub error: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenVideoConnection {
    pub ok: bool,
    pub error: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenStateConnection {
    pub ok: bool,
    pub error: String,
}
