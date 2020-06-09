extern crate gilrs;
extern crate image;

use crate::state;
use crate::utils;
use crate::video;

use druid::im::{vector, Vector};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::{
    lens,
    lens::LensExt,
    widget::{
        prelude::*, Align, Controller, CrossAxisAlignment, FillStrat, Flex, Label, List, Scroll,
        Split, Svg, ViewSwitcher, WidgetExt,
    },
    AppDelegate, Color, Command, Data, DelegateCtx, Env, ExtEventSink, KeyCode, Lens, Rect,
    Selector, Target, UnitPoint, Widget,
};
use druid_material_icons as icons;

use std::sync;
use std::thread;
use std::time;

// Base colors
pub const BASE_BG_COLOR: Color = Color::rgb8(0x1E, 0x1E, 0x1E);
pub const BASE_LIGHT_BG_COLOR: Color = Color::rgb8(0x33, 0x33, 0x33);
pub const BOTTOM_BAR_BG_COLOR: Color = Color::rgb8(0x00, 0x75, 0xC4);

pub const LIGHT_1_COLOR: Color = Color::rgb8(0xBD, 0xC3, 0xC7);
pub const LIGHT_2_COLOR: Color = Color::rgb8(0xE5, 0xE5, 0xE5);

pub enum ConnectionEvent {
    InitConnect,
    InitDisconnect,
    Error(String),
    Connected,
    Disconnected,
}

// Base events
pub const CONNECTION_COMMAND: Selector<ConnectionEvent> = Selector::new("connection.event");
pub const KEYBOARD_COMMAND: Selector<druid::Event> = Selector::new("keyboard.event");
pub const GAMEPAD_COMMAND: Selector<gilrs::EventType> = Selector::new("gamepad.event");

pub const RENDER_EVENT_COMMAND: Selector<image::RgbImage> = Selector::new("render.event");
pub const RENDER_SET_FPS_COMMAND: Selector<u8> = Selector::new("render.set.fps");
pub const STATE_EVENT_COMMAND: Selector = Selector::new("state.event");

#[derive(Clone, Default, Data, Lens)]
pub struct AppState {
    pub is_connected: bool,
    pub devices: Vector<String>,
    pub video_width: u16,
    pub video_height: u16,
    pub light_state: String,
    pub direction_state: String,
    pub connection_status: String,
    pub fps: u8,
}

impl AppState {
    pub fn default() -> Self {
        AppState {
            devices: vector!["device 1".to_string(), "device 2".to_string()],
            is_connected: false,
            video_width: 0,
            video_height: 0,
            light_state: "".to_string(),
            direction_state: "".to_string(),
            connection_status: "".to_string(),
            fps: 0,
        }
    }
}

pub struct Delegate {
    pub sink: ExtEventSink,
    video_conn: video::VideoConnection,
    state_conn: state::StateConnection,
    is_connected: sync::Arc<sync::atomic::AtomicBool>,
}

impl Delegate {
    pub fn new(
        sink: ExtEventSink,
        video_conn: video::VideoConnection,
        state_conn: state::StateConnection,
    ) -> Self {
        Delegate {
            sink: sink,
            video_conn: video_conn,
            state_conn: state_conn,
            is_connected: sync::Arc::new(sync::atomic::AtomicBool::default()),
        }
    }

    pub fn connect(&mut self) {
        let sink = self.sink.clone();

        self.state_conn.connect();
        match self.video_conn.connect() {
            Ok(_) => {
                sink.submit_command(CONNECTION_COMMAND, ConnectionEvent::Connected, None)
                    .expect("Failed to submit command");
            }
            Err(e) => {
                sink.submit_command(
                    CONNECTION_COMMAND,
                    ConnectionEvent::Error(format!("{}", e)),
                    None,
                )
                .expect("Failed to submit command");
                return;
            }
        };

        self.is_connected
            .clone()
            .store(true, sync::atomic::Ordering::Relaxed);
    }

    pub fn disconnect(&mut self) {
        self.video_conn.disconnect();
        self.state_conn.disconnect();

        self.is_connected
            .clone()
            .store(false, sync::atomic::Ordering::Relaxed);
    }

    pub fn on_connected(&self) {
        self.process_video_events();
        self.process_gamepad_events();
    }

    fn process_video_events(&self) {
        let sink = self.sink.clone();
        let mut fps_counter = video::FPSCounter::new(128);
        let connection = self.video_conn.connection().clone();

        thread::spawn(move || {
            info!("UI has been connected from the video stream");
            loop {
                match connection.clone().try_lock() {
                    Ok(rx_mutex) => match *rx_mutex {
                        Some(ref rx) => match rx.try_recv() {
                            Ok(image) => {
                                sink.submit_command(RENDER_EVENT_COMMAND, image.frame, None)
                                    .expect("Failed to submit command");
                                sink.submit_command(
                                    RENDER_SET_FPS_COMMAND,
                                    fps_counter.tick(),
                                    None,
                                )
                                .expect("Failed to submit command");
                            }
                            _ => {
                                thread::sleep(time::Duration::from_millis(40));
                            }
                        },
                        None => {
                            sink.submit_command(
                                CONNECTION_COMMAND,
                                ConnectionEvent::Disconnected,
                                None,
                            )
                            .expect("Failed to submit command");
                            break;
                        }
                    },
                    _ => {}
                }
            }
            info!("UI has been disconnected from the video stream");
        });
    }

    fn process_gamepad_events(&self) {
        let sink = self.sink.clone();
        let mut gamepad = match gilrs::Gilrs::new() {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to initialize a gamepad controller: {:?}", e);
                return;
            }
        };
        let is_connected = self.is_connected.clone();

        thread::spawn(move || loop {
            if !is_connected.load(sync::atomic::Ordering::Relaxed) {
                break;
            }
            match gamepad.next_event() {
                Some(gilrs::Event {
                    id: _,
                    event,
                    time: _,
                }) => sink
                    .submit_command(GAMEPAD_COMMAND, event.clone(), None)
                    .expect("Render command failed to submit."),
                None => {}
            }
            thread::sleep(time::Duration::from_millis(2));
        });
    }

    pub fn try_push_state(&mut self, data: &mut AppState) {
        match self.state_conn.push() {
            Some(ms) => {
                info!("Pushed the state.");
                let mut status = "";
                if ms.forward {
                    if ms.left {
                        status = "↖";
                    } else if ms.right {
                        status = "↗";
                    } else {
                        status = "⬆";
                    }
                } else if ms.backward {
                    if ms.left {
                        status = "↙";
                    } else if ms.right {
                        status = "↘";
                    } else {
                        status = "⬇";
                    }
                } else if ms.left {
                    status = "⬅";
                } else if ms.right {
                    status = "➡";
                }

                data.direction_state = status.to_string();
                data.light_state = if ms.lamp_enabled { "💡" } else { "" }.to_string();
            }
            _ => {}
        }
    }
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> bool {
        if cmd.is(KEYBOARD_COMMAND) | cmd.is(GAMEPAD_COMMAND) {
            if cmd.is(KEYBOARD_COMMAND) {
                match cmd.get_unchecked(KEYBOARD_COMMAND) {
                    Event::KeyDown(key) => match key.key_code {
                        KeyCode::KeyL => self.state_conn.enable_light(),
                        KeyCode::ArrowUp => self.state_conn.forward(),
                        KeyCode::ArrowDown => self.state_conn.backward(),
                        KeyCode::ArrowRight => self.state_conn.right(),
                        KeyCode::ArrowLeft => self.state_conn.left(),
                        _ => {}
                    },
                    Event::KeyUp(key) => match key.key_code {
                        KeyCode::KeyL => self.state_conn.disable_light(),
                        KeyCode::ArrowUp => self.state_conn.stop(),
                        KeyCode::ArrowDown => self.state_conn.stop(),
                        KeyCode::ArrowRight => self.state_conn.straight(),
                        KeyCode::ArrowLeft => self.state_conn.straight(),
                        _ => {}
                    },
                    _ => {}
                }
            }
            if cmd.is(GAMEPAD_COMMAND) {
                let gamepad_event = cmd.get_unchecked(GAMEPAD_COMMAND);
                match gamepad_event {
                    gilrs::EventType::ButtonPressed(button, _) => match button {
                        gilrs::Button::East => {
                            self.state_conn.enable_light();
                        }
                        _ => {}
                    },
                    gilrs::EventType::ButtonReleased(button, _) => match button {
                        gilrs::Button::East => self.state_conn.disable_light(),
                        _ => {}
                    },
                    gilrs::EventType::AxisChanged(axis, value, _) => match axis {
                        gilrs::Axis::LeftStickX => {
                            if value > &0.5 {
                                self.state_conn.right();
                            } else if value < &-0.5 {
                                self.state_conn.left();
                            } else {
                                self.state_conn.straight();
                            }
                        }
                        _ => {}
                    },
                    gilrs::EventType::ButtonChanged(button, value, _) => match button {
                        gilrs::Button::RightTrigger2 => {
                            if value > &0.5 {
                                self.state_conn.forward();
                            } else {
                                self.state_conn.stop();
                            }
                        }
                        gilrs::Button::LeftTrigger2 => {
                            if value > &0.5 {
                                self.state_conn.backward();
                            } else {
                                self.state_conn.stop();
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            self.try_push_state(data);
        }
        if cmd.is(CONNECTION_COMMAND) {
            match cmd.get_unchecked(CONNECTION_COMMAND) {
                ConnectionEvent::InitConnect => {
                    if !data.is_connected {
                        self.connect();
                    }
                }
                ConnectionEvent::InitDisconnect => {
                    if data.is_connected {
                        self.disconnect();
                    }
                }
                ConnectionEvent::Connected => {
                    data.is_connected = true;
                    self.on_connected();
                }
                ConnectionEvent::Disconnected => {
                    data.is_connected = false;
                }
                ConnectionEvent::Error(e) => {
                    data.is_connected = false;
                    data.connection_status = format!("{}", e);
                }
            }
        }
        true
    }
}

pub struct MovingImage {
    image_data: std::vec::Vec<u8>,
    size: Size,
    fill: FillStrat,
}

impl MovingImage {
    pub fn new() -> Self {
        MovingImage {
            image_data: vec![0; 0],
            size: Size::new(0.0, 0.0),
            fill: FillStrat::default(),
        }
    }

    pub fn fill_mode(mut self, mode: FillStrat) -> Self {
        self.fill = mode;
        self
    }
}

impl Widget<AppState> for MovingImage {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::Command(cmd) => {
                if cmd.is(RENDER_EVENT_COMMAND) {
                    let rgb_image = cmd.get_unchecked(RENDER_EVENT_COMMAND);
                    let sizeofimage = &rgb_image.dimensions();
                    self.image_data = rgb_image.to_vec();
                    self.size = Size::new(sizeofimage.0 as f64, sizeofimage.1 as f64);
                    ctx.request_paint();

                    data.video_height = sizeofimage.0 as u16;
                    data.video_width = sizeofimage.1 as u16;
                }
                if cmd.is(RENDER_SET_FPS_COMMAND) {
                    data.fps = cmd.get_unchecked(RENDER_SET_FPS_COMMAND).clone();
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppState,
        _env: &Env,
    ) -> Size {
        bc.debug_check("Image");

        if bc.is_width_bounded() {
            bc.max()
        } else {
            bc.constrain(self.size)
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &AppState, _env: &Env) {
        if self.image_data.len() == 0 {
            return;
        }

        // The ImageData's to_piet function does not clip to the image's size
        // CairoRenderContext is very like druids but with some extra goodies like clip
        if self.fill != FillStrat::Contain {
            let clip_rect = Rect::ZERO.with_size(ctx.size());
            ctx.clip(clip_rect);
        }

        let offset_matrix = self.fill.affine_to_fill(ctx.size(), self.size);
        ctx.transform(offset_matrix);
        let im = ctx
            .make_image(
                self.size.width as usize,
                self.size.height as usize,
                &self.image_data,
                ImageFormat::Rgb,
            )
            .unwrap();
        ctx.draw_image(&im, self.size.to_rect(), InterpolationMode::NearestNeighbor);
    }
}

struct FlexActionController {}

impl FlexActionController {
    pub fn new() -> Self {
        FlexActionController {}
    }
}

impl Controller<AppState, Flex<AppState>> for FlexActionController {
    fn event(
        &mut self,
        child: &mut Flex<AppState>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::KeyDown(_) => {
                ctx.submit_command(Command::new(KEYBOARD_COMMAND, event.clone()), None);
            }
            Event::KeyUp(_) => {
                ctx.submit_command(Command::new(KEYBOARD_COMMAND, event.clone()), None);
            }
            _ => {
                child.event(ctx, event, data, env);
            }
        }
    }
}

pub fn build_start_page() -> impl Widget<AppState> {
    let base_padding = 32.0;

    let mut base_col = Flex::column();
    let mut base_row = Flex::row();

    // Build header
    let mut header_row = Flex::row();
    let header_label = Label::new("RC.Machine")
        .with_text_color(LIGHT_1_COLOR)
        .with_text_size(36.0);
    header_row.add_flex_child(Align::left(header_label), 0.1);
    base_col.add_child(header_row.padding(base_padding));

    // Build left column
    let mut left_col: Flex<AppState> = Flex::column();

    // Add header
    left_col.add_flex_child(
        Flex::row()
            .with_flex_child(
                Align::left(
                    Label::new("Boards")
                        .with_text_color(LIGHT_2_COLOR)
                        .with_text_size(18.0),
                )
                .fix_height(30.0),
                0.7,
            )
            .with_flex_child(
                Align::right(
                    Label::new("+")
                        .on_click(|_, data: &mut AppState, _| {
                            let value = data.devices.len() + 1;
                            data.devices.push_back(format!("Test {}", value));
                        })
                        .fix_size(20.0, 20.0),
                ),
                0.3,
            ),
        0.1,
    );

    // Add boards list
    left_col.add_flex_child(
        Scroll::new(List::new(|| {
            Flex::row()
                .with_flex_child(
                    Label::new(|(_, item): &(Vector<String>, String), _env: &_| {
                        format!("List item #{}", item)
                    })
                    .align_vertical(UnitPoint::LEFT)
                    .padding(2.0)
                    .expand()
                    .height(30.0),
                    0.7,
                )
                .with_flex_child(
                    Align::right(
                        Flex::row()
                            .with_child(Label::new("connect").on_click(
                                |_ctx, (_, item): &mut (Vector<String>, String), _env| {
                                    println!("{:?}", item);
                                },
                            ))
                            .with_child(Label::new("-").on_click(
                                |_ctx, (shared, item): &mut (Vector<String>, String), _env| {
                                    shared.retain(|v| v != item);
                                },
                            )),
                    ),
                    0.3,
                )
        }))
        .vertical()
        .lens(lens::Id.map(
            |d: &AppState| (d.devices.clone(), d.devices.clone()),
            |d: &mut AppState, x: (Vector<String>, Vector<String>)| d.devices = x.0,
        )),
        1.0,
    );
    base_row.add_flex_child(Align::centered(left_col).padding(base_padding), 0.5);

    // Build right column
    let mut right_col: Flex<AppState> = Flex::column();
    let board = utils::load_svg("board.svg").unwrap();
    right_col.add_flex_child(
        Align::new(
            UnitPoint::BOTTOM_RIGHT,
            Svg::new(board)
                .border(LIGHT_2_COLOR, 0.0)
                .fix_size(500.0, 350.0),
        ),
        1.0,
    );
    base_row.add_flex_child(
        right_col.cross_axis_alignment(CrossAxisAlignment::Start),
        0.5,
    );
    base_col.add_flex_child(base_row, 0.1);

    base_col.background(BASE_BG_COLOR).debug_paint_layout()
}

pub fn build_main_page() -> impl Widget<AppState> {
    let mut col = Flex::column();

    // build left block
    let mut left_block = Flex::row();

    let connect_button = ViewSwitcher::new(
        |data: &AppState, _env| data.is_connected,
        |selector, _data, _env| match selector {
            true => Box::new(icons::CANCEL.new(Color::WHITE)),
            false => Box::new(icons::LINK.new(Color::WHITE)),
        },
    )
    .background(BOTTOM_BAR_BG_COLOR)
    .on_click(|ctx, data: &mut AppState, _env| {
        let conn_event = if data.is_connected {
            ConnectionEvent::InitDisconnect
        } else {
            ConnectionEvent::InitConnect
        };
        ctx.submit_command(Command::new(CONNECTION_COMMAND, conn_event), None);
        ctx.request_focus();
        ctx.set_active(true);
    })
    .padding(5.0);
    left_block.add_child(connect_button);
    left_block.add_child(Label::new(|d: &AppState, _: &Env| {
        format!("{}", d.connection_status)
    }));
    left_block.add_child(
        Align::centered(Label::new(|d: &AppState, _: &Env| {
            if d.is_connected {
                format!("{} FPS", d.fps)
            } else {
                "".to_string()
            }
        }))
        .fix_width(60.0),
    );
    left_block.add_child(
        Align::centered(Label::new(|d: &AppState, _: &Env| {
            if d.is_connected {
                format!("{} x {}", d.video_width, d.video_height)
            } else {
                "".to_string()
            }
        }))
        .fix_width(60.0),
    );

    // build right block
    let mut right_block = Flex::row();
    right_block.add_child(
        Align::centered(Label::new(|d: &AppState, _: &Env| {
            format!("{}", d.light_state)
        }))
        .fix_width(30.0),
    );
    right_block.add_child(
        Align::centered(Label::new(|d: &AppState, _: &Env| {
            format!("{}", d.direction_state)
        }))
        .fix_width(30.0),
    );

    // build bottom bar
    let footer_cols = Split::columns(Align::left(left_block), Align::right(right_block))
        .bar_size(0.0)
        .background(BOTTOM_BAR_BG_COLOR)
        .fix_height(30.0);

    // build frame window
    let video_view = ViewSwitcher::new(
        |data: &AppState, _env| data.is_connected,
        |selector, _data, _env| match selector {
            true => Box::new(
                MovingImage::new()
                    .fill_mode(FillStrat::Contain)
                    .background(BASE_BG_COLOR)
                    .center(),
            ),
            false => Box::new(icons::CAMERA.new(BASE_LIGHT_BG_COLOR)),
        },
    );

    col.add_flex_child(video_view, 1.0);
    col.add_flex_child(footer_cols, 0.0);

    col.controller(FlexActionController::new())
    //    .debug_paint_layout()
}
