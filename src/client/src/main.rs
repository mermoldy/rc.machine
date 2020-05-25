pub mod gamepad;
pub mod state;
pub mod video;
pub mod window;

#[macro_use]
extern crate log;
extern crate common;
extern crate simple_logger;

use common::settings;
use gamepad::Gamepad;
use state::RemoteState;
use video::{VideoFrame, VideoStream};
// use window::Window;

use std::sync::mpsc;
use std::thread;

fn main2() {
    println!("Initializing a logger...");
    env_logger::init();

    info!("Initializing a settings...");
    let settings = match settings::Settings::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize a settings: {:?}", e);
            std::process::exit(2);
        }
    };

    info!("Initializing a window...");
    // let mut app = match Window::new(settings.clone()) {
    //     Ok(r) => r,
    //     Err(e) => {
    //         error!("Failed to initialize a window: {:?}", e);
    //         std::process::exit(3);
    //     }
    // };

    info!("Initializing remote state connection...");
    let mut state = match RemoteState::new(settings.clone()) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize remote state: {:?}", e);
            std::process::exit(4);
        }
    };

    info!("Initializing a gamepad...");
    let mut gamepad = match Gamepad::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize a controller: {:?}", e);
            std::process::exit(5);
        }
    };

    info!("Initializing a video stream...");
    // app.set_log(
    //     format!(
    //         "Initializing a video stream ({}:{})...",
    //         settings.connection.host, settings.connection.video_port
    //     )
    //     .as_str(),
    // );
    let video_stream = VideoStream::new(settings.clone());
    let (tx, rx): (
        std::sync::mpsc::Sender<VideoFrame>,
        std::sync::mpsc::Receiver<VideoFrame>,
    ) = mpsc::channel();
    thread::spawn(move || match video_stream.connect(tx) {
        Ok(_) => {
            info!("A video stream initialized");
        }
        Err(e) => {
            error!("Failed to initialize a video stream: {:?}", e);
            std::process::exit(3);
        }
    });

    loop {
        // match rx.try_recv() {
        //     Ok(img) => {
        //         app.set_log("A video stream is established");
        //         // TODO: app.set_log("Rate = 50FPS | Size = 600 x 800 (100%) | Loss = 10%");
        //         app.set_image(img.data.rotate90());
        //     }
        //     _ => {}
        // }

        // match app.process_events() {
        //     window::Event::Exit => {
        //         info!("Exiting...");
        //         break;
        //     }
        //     window::Event::ButtonPressed(key) => match key {
        //         window::Key::L => state.enable_light(),
        //         window::Key::Up => state.forward(),
        //         window::Key::Down => state.backward(),
        //         window::Key::Right => state.right(),
        //         window::Key::Left => state.left(),
        //         _ => {}
        //     },
        //     window::Event::ButtonReleased(key) => match key {
        //         window::Key::L => state.disable_light(),
        //         window::Key::Up => state.stop(),
        //         window::Key::Down => state.stop(),
        //         window::Key::Right => state.straight(),
        //         window::Key::Left => state.straight(),
        //         _ => {}
        //     },
        //     _ => {}
        // }

        match gamepad.process_events() {
            gamepad::Event::ButtonPressed(button) => match button {
                gamepad::Button::East => state.enable_light(),
                _ => {}
            },
            gamepad::Event::ButtonReleased(button) => match button {
                gamepad::Button::East => state.disable_light(),
                _ => {}
            },
            gamepad::Event::AxisChanged(axis, value) => match axis {
                gamepad::Axis::LeftStickX => {
                    if value > 0.5 {
                        state.right();
                    } else if value < -0.5 {
                        state.left();
                    } else {
                        state.straight();
                    }
                }
                _ => {}
            },
            gamepad::Event::ButtonChanged(button, value) => match button {
                gamepad::Button::RightTrigger2 => {
                    if value > 0.5 {
                        state.forward();
                    } else {
                        state.stop();
                    }
                }
                gamepad::Button::LeftTrigger2 => {
                    if value > 0.5 {
                        state.backward();
                    } else {
                        state.stop();
                    }
                }
                _ => {}
            },
            _ => {}
        }

        match state.push() {
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

                let mut lamp_status = "light = off";
                if ms.lamp_enabled {
                    lamp_status = "light = on "
                }

                let mut engine_status = format!("state = off");
                if status != "" {
                    engine_status = format!("state =  {} ", status);
                }

                let heartbeat_status = format!("heartbeat = {}%", 100.0); // TODO

                // app.set_status(
                //     format!("{} | {} | {}", engine_status, lamp_status, heartbeat_status).as_str(),
                // );
            }
            _ => {}
        }
    }
    info!("Terminated");
}

extern crate image;
use druid::kurbo::BezPath;
use druid::kurbo::{Circle, Line};
use druid::piet::{FontBuilder, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget;
use druid::widget::prelude::*;
use druid::widget::{Align, Button, Container, Either, Label, Padding, Split};
use druid::{
    widget::{FillStrat, Flex, Image, ImageData, WidgetExt},
    AppLauncher, Color, Widget, WindowDesc,
};
use druid::{Affine, Rect};
use druid::{
    AppDelegate, Command, Data, DelegateCtx, Env, ExtEventSink, Lens, LocalizedString, Point,
    Selector, Target, Vec2,
};

use std::time;

struct MovingImage {
    image: Option<image::RgbImage>,
    size: Size,
    fill: FillStrat,
}

impl MovingImage {
    pub fn new() -> Self {
        MovingImage {
            image: None,
            size: Size::new(0.0, 0.0),
            fill: FillStrat::default(),
        }
    }
    /// A builder-style method for specifying the fill strategy.
    pub fn fill_mode(mut self, mode: FillStrat) -> Self {
        self.fill = mode;
        self
    }

    // pub fn start(mut self) {
    // }
    // pub fn set_image(&mut self, image: image::DynamicImage) {
    //     self.canvas = image.to_rgba();
    // }
}

const START_RENDER_COMMAND: Selector = Selector::new("start_render");
const RENDER_COMMAND: Selector = Selector::new("render");

// https://github.com/xi-editor/druid/blob/3a80b6373d7a6c6e6ebc36e0f37f6409ef180865/druid/examples/blocking_function.rs
impl Widget<AppState> for MovingImage {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::Command(cmd) => {
                if cmd.selector == RENDER_COMMAND {
                    println!("Render!..");
                    // match self.images.try_recv() {
                    //     Ok(img) => {
                    //         println!("Recv!..");
                    //         //self.image = Some(img.data.rotate90().to_rgb());
                    //         //&ctx.request_paint();
                    //         // self.data.
                    //         // self.paint(ctx: &mut PaintCtx, data: &AppState, _env: &Env)
                    //     }
                    //     _ => {}
                    // }
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
        println!("lifecycle");
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, data: &AppState, _env: &Env) {
        println!("Update");
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

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {
        println!("paint");
        let img = &mut self.image;

        match img {
            Some(rgb_image) => {
                // The ImageData's to_piet function does not clip to the image's size
                // CairoRenderContext is very like druids but with some extra goodies like clip
                if self.fill != FillStrat::Contain {
                    let clip_rect = Rect::ZERO.with_size(ctx.size());
                    ctx.clip(clip_rect);
                }
                let fill = &mut self.fill;

                let sizeofimage = &rgb_image.dimensions();
                let size = Size::new(sizeofimage.0 as f64, sizeofimage.1 as f64);
                self.size = size;

                ctx.with_save(|ctx| {
                    let offset_matrix = fill.affine_to_fill(ctx.size(), size);
                    ctx.transform(offset_matrix);
                    let im = ctx
                        .make_image(
                            size.width as usize,
                            size.height as usize,
                            &rgb_image.to_vec(),
                            ImageFormat::Rgb,
                        )
                        .unwrap();
                    ctx.draw_image(&im, size.to_rect(), InterpolationMode::Bilinear);
                });
            }
            _ => {}
        }
    }
}

struct Delegate {
    eventsink: ExtEventSink,
}

// struct RgbImage(image::RgbImage);

// impl Clone for RgbImage {
//     fn same(&self, other: &Self) -> bool {
//         return true;
//     }
// }

// impl Data for RgbImage {
//     fn same(&self, other: &Self) -> bool {
//         return true;
//     }
// }

struct Receiver(std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<video::VideoFrame>>>);

// impl Clone for Receiver {
//     fn clone(&self) -> Self {
//         self.clone()
//     }
// }
// impl Data for Receiver {
//     fn same(&self, other: &Self) -> bool {
//         true
//     }
// }

// impl Default for Receiver {
//     fn default() -> Self {
//         Receiver {}
//     }
// }

#[derive(Clone, Default, Data)]
struct AppState {
    processing: bool,
    value: u32,
}

fn listen_video_queue(sink: ExtEventSink) {
    thread::spawn(move || {
        info!("Initializing a settings...");
        let settings = match settings::Settings::new() {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to initialize a settings: {:?}", e);
                std::process::exit(2);
            }
        };
        let video_stream = VideoStream::new(settings.clone());
        let (tx, rx): (
            std::sync::mpsc::Sender<VideoFrame>,
            std::sync::mpsc::Receiver<VideoFrame>,
        ) = mpsc::channel();
        thread::spawn(move || match video_stream.connect(tx) {
            Ok(_) => {
                info!("A video stream initialized");
            }
            Err(e) => {
                error!("Failed to initialize a video stream: {:?}", e);
                std::process::exit(3);
            }
        });

        loop {
            match rx.try_recv() {
                Ok(img) => {
                    println!("Recv!..");
                    img.data.rotate90().to_rgb();
                    //&ctx.request_paint();
                    // self.data.
                    // self.paint(ctx: &mut PaintCtx, data: &AppState, _env: &Env)
                }
                _ => {}
            }
            // sink.submit_command(RENDER_COMMAND, 1, None)
            //     .expect("command failed to submit");
            thread::sleep(time::Duration::from_millis(120));
        }
    });
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
        match cmd.selector {
            START_RENDER_COMMAND => {
                data.processing = true;
                listen_video_queue(self.eventsink.clone());
                true
            }
            // FINISH_SLOW_FUNCTION => {
            //     data.processing = false;
            //     let number = cmd.get_object::<u32>().expect("api violation");
            //     data.value = *number;
            //     true
            // }
            _ => true,
        }
    }
}

pub fn main3() {
    println!("Initializing a logger...");
    simple_logger::init();

    info!("Initializing a settings...");
    let settings = match settings::Settings::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize a settings: {:?}", e);
            std::process::exit(2);
        }
    };

    info!("Initializing a video stream...");
    // app.set_log(
    //     format!(
    //         "Initializing a video stream ({}:{})...",
    //         settings.connection.host, settings.connection.video_port
    //     )
    //     .as_str(),
    // );

    // fn ui_builder() -> impl Widget<AppState> {
    //     // let image_data = image::load_from_memory(include_bytes!("img.png"))
    //     //     .map_err(|e| e)
    //     //     .unwrap();
    // };

    let main_window = WindowDesc::new(|| {
        let mut col = Flex::column();

        let button = Button::new("Start slow increment")
            .on_click(|ctx, _data: &mut AppState, _env| {
                let cmd = Command::new(START_RENDER_COMMAND, 0);
                ctx.submit_command(cmd, None);
            })
            .padding(5.0);

        let mut mv = MovingImage::new();
        col.add_flex_child(
            mv.fill_mode(FillStrat::FitWidth)
                .border(Color::WHITE, 1.0)
                .center(),
            1.0,
        );
        // thread::spawn(move || {
        //     match &mv.images.try_recv() {
        //         Ok(img) => {
        //             println!("Recv!..");
        //             //mv.image = Some(img.data.rotate90().to_rgb());
        //             //mv.render();
        //             // &ctx.request_paint();
        //             // self.data.
        //             // self.paint(ctx: &mut PaintCtx, data: &AppState, _env: &Env)
        //         }
        //         _ => {}
        //     }
        //     thread::sleep(time::Duration::from_millis(40));
        // });
        let fixed_cols = Padding::new(
            10.0,
            Container::new(
                Split::columns(
                    Align::centered(button),
                    Align::centered(Label::new("Right Split")),
                )
                .split_point(0.5),
            )
            .border(Color::WHITE, 1.0),
        )
        .fix_height(60.0);
        col.add_flex_child(fixed_cols, 0.0);

        col
    });

    let app = AppLauncher::with_window(main_window);
    let delegate = Delegate {
        eventsink: app.get_external_handle(),
    };
    app.delegate(delegate)
        .launch(AppState::default())
        .expect("launch failed");
}

fn main() {
    info!("Initializing a settings...");
    let settings = match settings::Settings::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize a settings: {:?}", e);
            std::process::exit(2);
        }
    };
    let video_stream = VideoStream::new(settings.clone());
    let (tx, rx): (
        std::sync::mpsc::Sender<VideoFrame>,
        std::sync::mpsc::Receiver<VideoFrame>,
    ) = mpsc::channel();
    thread::spawn(move || match video_stream.connect(tx) {
        Ok(_) => {
            info!("A video stream initialized");
        }
        Err(e) => {
            error!("Failed to initialize a video stream: {:?}", e);
            std::process::exit(3);
        }
    });

    loop {
        match rx.try_recv() {
            Ok(img) => {
                println!("Recv!..");
                img.data.rotate90().to_rgba();
                //&ctx.request_paint();
                // self.data.
                // self.paint(ctx: &mut PaintCtx, data: &AppState, _env: &Env)
            }
            _ => {}
        }
        // sink.submit_command(RENDER_COMMAND, 1, None)
        //     .expect("command failed to submit");
        thread::sleep(time::Duration::from_millis(120));
    }
}
