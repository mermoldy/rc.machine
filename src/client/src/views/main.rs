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
                if cmd.selector == RENDER_COMMAND {
                    let rgb_image = cmd.get_object::<image::RgbImage>().expect("api violation");
                    let sizeofimage = &rgb_image.dimensions();
                    self.image_data = rgb_image.to_vec();
                    self.size = Size::new(sizeofimage.0 as f64, sizeofimage.1 as f64);
                    ctx.request_paint();
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

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, data: &AppState, _env: &Env) {}

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

fn listen_video_queue(sink: ExtEventSink) {
    info!("Initializing a video stream...");
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
                    sink.submit_command(RENDER_COMMAND, img.frame, None)
                        .expect("Render command failed to submit.");
                }
                _ => {}
            }
            thread::sleep(time::Duration::from_millis(20));
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
            _ => true,
        }
    }
}
const VIDEO_BG_COLOR: Color = Color::rgb8(0x1E, 0x1E, 0x1E);
const BOTTOM_BAR_BG_COLOR: Color = Color::rgb8(0x00, 0x75, 0xC4);

fn build_ui() -> impl Widget<AppState> {
    let mut col = Flex::column();

    // build left block
    let mut left_block = Flex::row();
    let connect_button = Button::new("Connect 🎥")
        .background(BOTTOM_BAR_BG_COLOR)
        .on_click(|ctx, _data: &mut AppState, _env| {
            let cmd = Command::new(START_RENDER_COMMAND, 0);
            ctx.submit_command(cmd, None);
        })
        .padding(5.0);
    left_block.add_child(Align::left(connect_button));
    left_block.add_child(Align::left(Label::new("50FPS")));
    left_block.add_child(Align::left(Label::new("600 x 800 (100%)")));
    left_block.add_child(Align::left(Label::new("10% loss")));

    // build right block
    let mut right_block = Flex::row();
    right_block.add_child(Align::left(Label::new("💡")));
    right_block.add_child(Align::left(Label::new("⬅️")));
    right_block.add_child(Align::left(Label::new(format!("❤️ ({}%)", 100.0))));

    // build bottom bar
    let footer_cols = Padding::new(
        0.0,
        Container::new(
            Split::columns(Align::left(left_block), Align::right(right_block)).split_point(0.5),
        )
        .background(BOTTOM_BAR_BG_COLOR),
    )
    .fix_height(30.0);

    // build frame window
    col.add_flex_child(
        MovingImage::new()
            .fill_mode(FillStrat::FitWidth)
            .background(VIDEO_BG_COLOR)
            .center(),
        1.0,
    );
    col.add_flex_child(footer_cols, 0.0);

    //col.debug_paint_layout()
    col
}
