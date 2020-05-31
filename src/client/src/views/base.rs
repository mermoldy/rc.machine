use druid::{AppDelegate, Color, Command, Data, DelegateCtx, Env, ExtEventSink, Selector, Target};

// Base colors
pub const BASE_BG_COLOR: Color = Color::rgb8(0x1E, 0x1E, 0x1E);
pub const BOTTOM_BAR_BG_COLOR: Color = Color::rgb8(0x00, 0x75, 0xC4);

pub const LIGHT_1_COLOR: Color = Color::rgb8(0xBD, 0xC3, 0xC7);
pub const LIGHT_2_COLOR: Color = Color::rgb8(0xE5, 0xE5, 0xE5);

// Base events
const START_RENDER_COMMAND: Selector = Selector::new("start_render");
const RENDER_COMMAND: Selector = Selector::new("render");

#[derive(Clone, Default, Data)]
pub struct AppState {
    processing: bool,
    value: u32,
}

pub struct Delegate {
    pub eventsink: ExtEventSink,
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        _data: &mut AppState,
        _env: &Env,
    ) -> bool {
        match cmd.selector {
            START_RENDER_COMMAND => {
                info!("render");
                true
            }
            _ => true,
        }
    }
}
