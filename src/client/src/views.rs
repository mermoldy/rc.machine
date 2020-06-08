extern crate image;

use crate::gamepad;
use crate::state::RemoteState;
use crate::utils;
use crate::video;
use video::{VideoFrame, VideoStream};

use common::settings;

use druid::im::{vector, Vector};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::{
    lens,
    lens::LensExt,
    widget::{
        prelude::*, Align, Button, Container, Controller, CrossAxisAlignment, FillStrat, Flex,
        Label, List, Padding, Scroll, Split, WidgetExt,
    },
    AppDelegate, Color, Command, Data, DelegateCtx, Env, ExtEventSink, KeyCode, Lens, Rect,
    Selector, Target, UnitPoint, Widget,
};

use std::sync::mpsc;
use std::thread;
use std::time;

// Base colors
pub const BASE_BG_COLOR: Color = Color::rgb8(0x1E, 0x1E, 0x1E);
pub const BOTTOM_BAR_BG_COLOR: Color = Color::rgb8(0x00, 0x75, 0xC4);

pub const LIGHT_1_COLOR: Color = Color::rgb8(0xBD, 0xC3, 0xC7);
pub const LIGHT_2_COLOR: Color = Color::rgb8(0xE5, 0xE5, 0xE5);

// Base events
pub const RENDER_START_COMMAND: Selector = Selector::new("render.start");
pub const RENDER_EVENT_COMMAND: Selector<image::RgbImage> = Selector::new("render.event");
pub const RENDER_SET_FPS_COMMAND: Selector<u8> = Selector::new("render.set.fps");
pub const GAMEPAD_EVENT_COMMAND: Selector<gamepad::Event> = Selector::new("gamepad.event");
pub const STATE_EVENT_COMMAND: Selector = Selector::new("state.event");

#[derive(Clone, Default, Data, Lens)]
pub struct AppState {
    pub is_connected: bool,
    pub devices: Vector<String>,
    pub video_width: u16,
    pub video_height: u16,
    pub light_state: String,
    pub direction_state: String,
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
            fps: 0,
        }
    }
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
        data: &mut AppState,
        _env: &Env,
    ) -> bool {
        if cmd.is(RENDER_START_COMMAND) {
            if !data.is_connected {
                info!("Initializing a settings...");
                let settings = match settings::Settings::new() {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Failed to initialize a settings: {:?}", e);
                        std::process::exit(2);
                    }
                };

                info!("Initializing a gamepad...");
                let mut gpad = match gamepad::Gamepad::new() {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Failed to initialize a controller: {:?}", e);
                        std::process::exit(5);
                    }
                };

                let video_stream = VideoStream::new(settings.clone());
                let (tx, rx): (mpsc::Sender<VideoFrame>, mpsc::Receiver<VideoFrame>) =
                    mpsc::channel();

                thread::spawn(move || match video_stream.connect(tx) {
                    Ok(_) => {
                        info!("A video stream initialized");
                        return false;
                    }
                    Err(e) => {
                        error!("Failed to initialize a video stream: {:?}", e);
                        return false;
                    }
                });

                info!("Initializing a video stream...");
                let sink = self.eventsink.clone();
                let mut fps_counter = utils::FPSCounter::new(128);

                thread::spawn(move || loop {
                    match rx.try_recv() {
                        Ok(img) => {
                            sink.submit_command(RENDER_EVENT_COMMAND, img.frame, None)
                                .expect("Render command failed to submit.");
                            sink.submit_command(RENDER_SET_FPS_COMMAND, fps_counter.tick(), None)
                                .expect("Render command failed to submit.");
                        }
                        _ => {}
                    }

                    let gpad_event = gpad.process_events();
                    match gpad_event {
                        gamepad::Event::None => {}
                        _ => {
                            sink.submit_command(GAMEPAD_EVENT_COMMAND, gpad_event, None)
                                .expect("Render command failed to submit.");
                        }
                    }

                    thread::sleep(time::Duration::from_millis(40));
                });

                data.is_connected = true;
                false
            } else {
                true
            }
        } else {
            true
        }
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

struct FlexActionController {
    state: RemoteState,
}

impl FlexActionController {
    pub fn new() -> Self {
        info!("Initializing remote state connection...");

        info!("Initializing a settings...");
        let settings = match settings::Settings::new() {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to initialize a settings: {:?}", e);
                std::process::exit(2);
            }
        };

        let state = match RemoteState::new(settings.clone()) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to initialize remote state: {:?}", e);
                std::process::exit(4);
            }
        };

        FlexActionController { state: state }
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
            Event::KeyDown(key) => match key.key_code {
                KeyCode::KeyL => self.state.enable_light(),
                KeyCode::ArrowUp => self.state.forward(),
                KeyCode::ArrowDown => self.state.backward(),
                KeyCode::ArrowRight => self.state.right(),
                KeyCode::ArrowLeft => self.state.left(),
                _ => {}
            },
            Event::KeyUp(key) => match key.key_code {
                KeyCode::KeyL => self.state.disable_light(),
                KeyCode::ArrowUp => self.state.stop(),
                KeyCode::ArrowDown => self.state.stop(),
                KeyCode::ArrowRight => self.state.straight(),
                KeyCode::ArrowLeft => self.state.straight(),
                _ => {}
            },
            Event::Command(cmd) => {
                if cmd.is(GAMEPAD_EVENT_COMMAND) {
                    let gamepad_event = cmd.get_unchecked(GAMEPAD_EVENT_COMMAND);
                    match gamepad_event {
                        gamepad::Event::ButtonPressed(button) => match button {
                            gamepad::Button::East => self.state.enable_light(),
                            _ => {}
                        },
                        gamepad::Event::ButtonReleased(button) => match button {
                            gamepad::Button::East => self.state.disable_light(),
                            _ => {}
                        },
                        gamepad::Event::AxisChanged(axis, value) => match axis {
                            gamepad::Axis::LeftStickX => {
                                if value > &0.5 {
                                    self.state.right();
                                } else if value < &-0.5 {
                                    self.state.left();
                                } else {
                                    self.state.straight();
                                }
                            }
                            _ => {}
                        },
                        gamepad::Event::ButtonChanged(button, value) => match button {
                            gamepad::Button::RightTrigger2 => {
                                if value > &0.5 {
                                    self.state.forward();
                                } else {
                                    self.state.stop();
                                }
                            }
                            gamepad::Button::LeftTrigger2 => {
                                if value > &0.5 {
                                    self.state.backward();
                                } else {
                                    self.state.stop();
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                } else {
                    child.event(ctx, event, data, env)
                }
            }
            _ => child.event(ctx, event, data, env),
        }

        match self.state.push() {
            Some(ms) => {
                info!("Pushed the state.");

                if ms.lamp_enabled {
                    data.light_state = "💡".to_string();
                } else {
                    data.light_state = "".to_string();
                }

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
            }
            _ => {}
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
    let right_row: Flex<AppState> = Flex::column();
    let mut right_col: Flex<AppState> = Flex::column();

    right_col.add_flex_child(right_row.padding(base_padding), 0.5);

    // let board = utils::load_svg("board.svg").unwrap();
    // right_col.add_flex_child(
    //     Align::new(
    //         UnitPoint::BOTTOM_RIGHT,
    //         Svg::new(board)
    //             .border(LIGHT_2_COLOR, 0.0)
    //             .fix_size(500.0, 350.0),
    //     ),
    //     1.0,
    // );
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
    let connect_button = Button::new("🎥")
        .background(BOTTOM_BAR_BG_COLOR)
        .on_click(|ctx, _data: &mut AppState, _env| {
            let cmd = Command::new(RENDER_START_COMMAND, ());
            ctx.submit_command(cmd, None);

            ctx.request_focus();
            ctx.set_active(true);
        })
        .padding(5.0);
    left_block.add_child(Align::left(connect_button));
    left_block.add_child(Label::new(|d: &AppState, _: &Env| format!("{} FPS", d.fps)));
    left_block.add_child(Label::new(|d: &AppState, _: &Env| {
        format!("{} x {}", d.video_width, d.video_height)
    }));

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
    col.add_flex_child(
        MovingImage::new()
            .fill_mode(FillStrat::Contain)
            .background(BASE_BG_COLOR)
            .center(),
        1.0,
    );
    //col.add_flex_child(TextBox::new().lens(AppState::text), 1.0);
    col.add_flex_child(footer_cols, 0.0);

    col.controller(FlexActionController::new())
    //    .debug_paint_layout()
    // col
}
