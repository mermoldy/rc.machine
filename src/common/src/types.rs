extern crate serde;

use self::serde::{Deserialize, Serialize};
use settings::{Heartbeat, Video};
use std::fmt;

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientHello {
    pub token: String,
    pub video: Video,
    pub heartbeat: Heartbeat,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerHello {
    pub ok: bool,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct MachineState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub lamp_enabled: bool,
}

impl PartialEq for MachineState {
    fn eq(&self, other: &Self) -> bool {
        (self.forward == other.forward)
            && (self.backward == other.backward)
            && (self.left == other.left)
            && (self.right == other.right)
            && (self.lamp_enabled == other.lamp_enabled)
    }
}
impl fmt::Debug for MachineState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MachineState(forward={:?}, backward={:?}, left={:?}, right={:?}, lamp_enabled={:?})",
            self.forward, self.backward, self.left, self.right, self.lamp_enabled,
        )
    }
}
impl Eq for MachineState {}

impl MachineState {
    pub fn new() -> MachineState {
        MachineState {
            forward: false,
            backward: false,
            left: false,
            right: false,
            lamp_enabled: false,
        }
    }
}
