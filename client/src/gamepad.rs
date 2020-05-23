extern crate gilrs;

use self::gilrs::{Event as GEvent, EventType, Gilrs};

pub use self::gilrs::{Axis, Button};

use std::error::Error;

pub struct Gamepad {
    gilrs: Gilrs,
}

pub enum Event {
    ButtonPressed(Button),
    ButtonReleased(Button),
    AxisChanged(Axis, f32),
    ButtonChanged(Button, f32),
    None,
}

impl Gamepad {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let gilrs = Gilrs::new()?;

        Ok(Gamepad { gilrs: gilrs })
    }

    pub fn process_events(&mut self) -> Event {
        if let Some(GEvent {
            id: _,
            event,
            time: _,
        }) = self.gilrs.next_event()
        {
            match event {
                EventType::ButtonPressed(button, _) => {
                    return Event::ButtonPressed(button);
                }
                EventType::ButtonReleased(button, _) => {
                    return Event::ButtonReleased(button);
                }
                EventType::AxisChanged(axis, value, _) => {
                    return Event::AxisChanged(axis, value);
                }
                EventType::ButtonChanged(button, value, _) => {
                    return Event::ButtonChanged(button, value);
                }
                _ => {}
            }
        };
        Event::None
    }
}
