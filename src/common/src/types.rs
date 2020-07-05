extern crate image;
extern crate serde;

use self::serde::{Deserialize, Serialize};
use settings::{Heartbeat, Video};
use std::fmt;

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

pub enum MachineEvents {
    Forward,
    Backward,
    Stop,

    Left,
    Right,
    Straight,

    LightTrigger,
}

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

    pub fn update(&mut self, event: MachineEvents) -> bool {
        match event {
            MachineEvents::Forward => {
                if !self.forward {
                    self.forward = true;
                    true
                } else {
                    false
                }
            }
            MachineEvents::Backward => {
                if !self.backward {
                    self.backward = true;
                    true
                } else {
                    false
                }
            }
            MachineEvents::Stop => {
                if self.forward || self.backward {
                    self.forward = false;
                    self.backward = false;
                    true
                } else {
                    false
                }
            }
            MachineEvents::Left => {
                if !self.left {
                    self.left = true;
                    true
                } else {
                    false
                }
            }
            MachineEvents::Right => {
                if !self.right {
                    self.right = true;
                    true
                } else {
                    false
                }
            }
            MachineEvents::Straight => {
                if self.left || self.right {
                    self.left = false;
                    self.right = false;
                    true
                } else {
                    false
                }
            }

            MachineEvents::LightTrigger => {
                self.lamp_enabled = !self.lamp_enabled;
                true
            }
        }
    }
}

pub struct VideoFrame {
    pub image: image::RgbImage,
}
