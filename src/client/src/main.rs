pub mod conn;
pub mod utils;
pub mod views;

#[macro_use]
extern crate log;
extern crate common;
extern crate simple_error;
extern crate stoppable_thread;

extern crate env_logger;

use common::settings;
use druid::{AppLauncher, LocalizedString, WindowDesc};

pub fn main() {
    println!("Initializing a logger...");
    env_logger::init();

    info!("Initializing a settings...");
    let settings = match settings::Settings::new() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to initialize a settings: {:?}", e);
            std::process::exit(1);
        }
    };

    let main_window = WindowDesc::new(views::build_ui)
        .title(LocalizedString::new("app-title").with_placeholder("RC.Machine"))
        .window_size((700.0, 540.0))
        .with_min_size((700.0, 540.0));

    let app = AppLauncher::with_window(main_window);
    let delegate = views::Delegate::new(app.get_external_handle(), settings);

    info!("Initializing application window...");
    app.delegate(delegate)
        .launch(views::AppState::default())
        .expect("Application launch failed.");
}
