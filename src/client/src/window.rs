// extern crate find_folder;
// extern crate gfx_device_gl;
// extern crate image;
// extern crate piston;
// extern crate piston_window;
// use self::piston::input;
// use self::piston::window::Window as PWindow;
// use self::piston_window::{
//     CloseEvent, EventLoop, PressEvent, ReleaseEvent, RenderEvent, Transformed,
// };
// use crate::settings;

// use std::io::Error as IOError;
// use std::io::ErrorKind as IOErrorKind;

// use std::error::Error;

// pub use piston::Key;

// pub struct Window<W: PWindow = piston_window::PistonWindow> {
//     window: W,
//     events: piston_window::Events,

//     texture_context: piston_window::TextureContext<
//         gfx_device_gl::Factory,
//         gfx_device_gl::Resources,
//         gfx_device_gl::CommandBuffer,
//     >,
//     canvas: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>,
//     texture: piston_window::G2dTexture,
//     glyphs: piston_window::Glyphs,
//     log: String,
//     status: String,

//     width: u32,
//     height: u32,
// }

// pub enum Event {
//     Rendered,
//     Exit,
//     ButtonPressed(Key),
//     ButtonReleased(Key),
//     None,
// }

// impl Window {
//     pub fn new(settings: settings::Settings) -> Result<Self, Box<dyn Error>> {
//         let (height, width) = settings.video.resolution;

//         let mut window: piston_window::PistonWindow =
//             piston_window::WindowSettings::new("RC.Machine", [width, height])
//                 .exit_on_esc(true)
//                 .resizable(false)
//                 .exit_on_esc(true)
//                 .build()?;

//         let ttf_assets =
//             match find_folder::Search::ParentsThenKids(1, 4).for_folder("assets/ttf/firacode") {
//                 Ok(res) => res,
//                 Err(err) => {
//                     return Err(Box::new(IOError::new(
//                         IOErrorKind::Other,
//                         format!("Failed to load fonts directory: {}", err),
//                     )))
//                 }
//             };
//         let glyphs = match window.load_font(ttf_assets.join("FiraCode-Medium.ttf")) {
//             Ok(res) => res,
//             Err(err) => {
//                 return Err(Box::new(IOError::new(
//                     IOErrorKind::Other,
//                     format!("Failed to load fonts: {}", err),
//                 )))
//             }
//         };

//         let canvas = image::ImageBuffer::new(width as u32, height as u32);
//         let mut texture_context = piston_window::TextureContext {
//             factory: window.factory.clone(),
//             encoder: window.factory.create_command_buffer().into(),
//         };
//         let texture: piston_window::G2dTexture = piston_window::Texture::from_image(
//             &mut texture_context,
//             &canvas,
//             &piston_window::TextureSettings::new(),
//         )?;
//         let events = piston_window::Events::new(piston_window::EventSettings::new().lazy(false));

//         Ok(Window {
//             window: window,
//             events: events,
//             texture: texture,
//             texture_context: texture_context,
//             canvas: canvas,
//             glyphs: glyphs,
//             width: width,
//             height: height,
//             log: String::from(""),
//             status: String::from(""),
//         })
//     }

//     pub fn set_image(&mut self, image: image::DynamicImage) {
//         self.canvas = image.to_rgba();
//     }

//     pub fn set_log(&mut self, text: &str) {
//         self.log = String::from(text);
//     }

//     pub fn set_status(&mut self, text: &str) {
//         self.status = String::from(text);
//     }

//     pub fn process_events(&mut self) -> Event {
//         return match self.events.next(&mut self.window) {
//             Some(e) => {
//                 if let Some(_) = e.close_args() {
//                     return Event::Exit;
//                 }
//                 if let Some(_) = e.render_args() {
//                     match self.draw(e) {
//                         Ok(_) => {}
//                         Err(e) => {
//                             error!("Failed to draw a frame: {:?}", e);
//                         }
//                     };
//                     return Event::Rendered;
//                 }
//                 if let Some(input::Button::Keyboard(key)) = e.press_args() {
//                     if key == input::Key::Escape || key == input::Key::Q {
//                         return Event::Exit;
//                     }
//                     return Event::ButtonPressed(key);
//                 }
//                 if let Some(input::Button::Keyboard(key)) = e.release_args() {
//                     return Event::ButtonReleased(key);
//                 }

//                 Event::None
//             }
//             None => Event::None,
//         };
//     }

//     fn draw(&mut self, e: piston_window::Event) -> Result<(), Box<dyn Error>> {
//         let window = &mut self.window;
//         let texture = &mut self.texture;
//         let glyphs = &mut self.glyphs;
//         let texture_context = &mut self.texture_context;
//         let log = &mut self.log;
//         let status = &mut self.status;

//         let width = window.size().width;
//         let height = window.size().height;

//         let initial_width = self.width;
//         let initial_height = self.height;

//         &texture.update(texture_context, &self.canvas)?;

//         window.draw_2d(&e, |c, g, device| {
//             texture_context.encoder.flush(device);
//             piston_window::clear([0.11; 4], g);

//             piston_window::image(
//                 texture,
//                 c.transform.trans(0.0, 0.0).scale(
//                     1.0 * (width / initial_width as f64),
//                     1.0 * (height / initial_height as f64),
//                 ),
//                 g,
//             );

//             piston_window::rectangle(
//                 [0.5, 0.5, 0.5, 1.0],
//                 [0.0, height - 30.0, width, height - 29.5],
//                 c.transform,
//                 g,
//             );
//             piston_window::rectangle(
//                 [0.15, 0.15, 0.15, 1.0],
//                 [0.0, height - 29.4, width, height],
//                 c.transform,
//                 g,
//             );

//             let log_transform = c.transform.trans(10.0, height - 10.0);
//             piston_window::text::Text::new_color([0.1, 0.8, 0.46, 1.0], 12)
//                 .draw(&log.as_str(), glyphs, &c.draw_state, log_transform, g)
//                 .unwrap();

//             let state_transform = c.transform.trans(width - 368.0, height - 10.0);
//             piston_window::text::Text::new_color([0.1, 0.8, 0.46, 1.0], 12)
//                 .draw(&status.as_str(), glyphs, &c.draw_state, state_transform, g)
//                 .unwrap();

//             // Update glyphs before rendering.
//             glyphs.factory.encoder.flush(device);
//         });

//         Ok(())
//     }
// }
