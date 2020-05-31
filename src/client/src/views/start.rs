extern crate image;

use crate::utils;
use crate::views::base::*;
use crate::views::*;

use druid::{
    theme,
    widget::{
        Align, Button, Checkbox, Container, CrossAxisAlignment, Either, FillStrat, Flex,
        FlexParams, Image, ImageData, Label, MainAxisAlignment, Padding, Painter, Parse, Scroll,
        Split, Stepper, Svg, TextBox, WidgetExt,
    },
    AppLauncher, Color, Data, Key, Lens, LensExt, LensWrap, LocalizedString, PlatformError,
    RenderContext, UnitPoint, Widget, WindowDesc,
};
use std::fmt::Display;

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

    let boards_label = Label::new(|data: &AppState, _env: &_| format!("{}", "Boards"))
        .with_text_color(LIGHT_2_COLOR)
        .with_text_size(18.0)
        .on_click(|_, data, _| {});
    left_col.add_flex_child(Align::left(boards_label).fix_height(30.0), 0.1);
    let mut boards_list = Flex::column();
    for i in 0..100 {
        let mut button_row = Flex::row();
        button_row.add_child(Label::new(format!("Board #{}", i)));
        boards_list.add_child(Align::left(button_row));
    }
    left_col.add_child(Align::centered(Scroll::new(boards_list)));
    base_row.add_flex_child(Align::centered(left_col).padding(base_padding), 0.5);

    // Build right column
    let mut right_row: Flex<AppState> = Flex::column();
    let mut right_col: Flex<AppState> = Flex::column();

    let board = utils::load_svg("board.svg").unwrap();

    let input_devices_label =
        Label::new(|data: &AppState, _env: &_| format!("{}", "Available input devices"))
            .with_text_color(LIGHT_2_COLOR)
            .with_text_size(18.0)
            .on_click(|_, data, _| {});
    let mut input_devices_list = Flex::column();

    right_row.add_flex_child(Align::left(input_devices_label).fix_height(30.0), 0.1);
    right_row.add_flex_child(Align::centered(input_devices_list), 0.1);
    right_col.add_flex_child(right_row.padding(base_padding), 0.5);

    right_col.add_flex_child(
        Align::new(
            UnitPoint::BOTTOM_RIGHT,
            Svg::new(board)
                .border(LIGHT_2_COLOR, 0.0)
                .fix_size(500.0, 350.0),
        ),
        1.0,
    );
    base_row.add_flex_child(right_col.padding(0.0), 0.5);
    base_col.add_flex_child(base_row, 0.1);

    base_col.background(BASE_BG_COLOR) //.debug_paint_layout()
}
