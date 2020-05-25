extern crate find_folder;
extern crate gfx_device_gl;
extern crate image;
extern crate piston;
extern crate piston_window;
use self::piston::input;
use self::piston::window::Window as PWindow;
use self::piston_window::{
    CloseEvent, EventLoop, PressEvent, ReleaseEvent, RenderEvent, Transformed,
};

use std::error::Error;

pub use piston::Key;

pub struct Window<W: PWindow = piston_window::PistonWindow> {
    window: W,
    events: piston_window::Events,

    texture_context: piston_window::TextureContext<
        gfx_device_gl::Factory,
        gfx_device_gl::Resources,
        gfx_device_gl::CommandBuffer,
    >,
    canvas: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>,
    texture: piston_window::G2dTexture,
    glyphs: piston_window::Glyphs,
    log: String,

    width: f64,
    height: f64,
}

pub enum Event {
    Rendered,
    Exit,
    ButtonPressed(Key),
    ButtonReleased(Key),
    None,
}

impl Window {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let width = 800.0;
        let height = 600.0;

        let mut window: piston_window::PistonWindow =
            piston_window::WindowSettings::new("RC.Machine", [width, height])
                .exit_on_esc(true)
                .resizable(false)
                .exit_on_esc(true)
                .build()?;

        let ttf_assets =
            find_folder::Search::ParentsThenKids(1, 4).for_folder("assets/ttf/firacode")?;
        let glyphs = window.load_font(ttf_assets.join("FiraCode-Medium.ttf"))?;

        let canvas = image::ImageBuffer::new(width as u32, height as u32);
        let mut texture_context = piston_window::TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into(),
        };
        let texture: piston_window::G2dTexture = piston_window::Texture::from_image(
            &mut texture_context,
            &canvas,
            &piston_window::TextureSettings::new(),
        )?;
        let events = piston_window::Events::new(piston_window::EventSettings::new().lazy(false));

        Ok(Window {
            window: window,
            events: events,
            texture: texture,
            texture_context: texture_context,
            canvas: canvas,
            glyphs: glyphs,
            width: width,
            height: height,
            log: String::from(""),
        })
    }

    pub fn set_image(&mut self, image: image::DynamicImage) {
        self.canvas = image.to_rgba();
    }

    pub fn set_log(&mut self, text: &str) {
        self.log = String::from(text);
    }

    pub fn process_events(&mut self) -> Event {
        return match self.events.next(&mut self.window) {
            Some(e) => {
                if let Some(_) = e.close_args() {
                    return Event::Exit;
                }
                if let Some(_) = e.render_args() {
                    match self.draw(e) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to draw a frame: {:?}", e);
                        }
                    };
                    return Event::Rendered;
                }
                if let Some(input::Button::Keyboard(key)) = e.press_args() {
                    if key == input::Key::Escape || key == input::Key::Q {
                        return Event::Exit;
                    }
                    return Event::ButtonPressed(key);
                }
                if let Some(input::Button::Keyboard(key)) = e.release_args() {
                    return Event::ButtonReleased(key);
                }

                Event::None
            }
            None => Event::None,
        };
    }

    fn draw(&mut self, e: piston_window::Event) -> Result<(), Box<dyn Error>> {
        let window = &mut self.window;
        let texture = &mut self.texture;
        let glyphs = &mut self.glyphs;
        let texture_context = &mut self.texture_context;
        let log = &mut self.log;

        let width = window.size().width;
        let height = window.size().height;

        let initial_width = self.width;
        let initial_height = self.height;

        &texture.update(texture_context, &self.canvas)?;

        window.draw_2d(&e, |c, g, device| {
            texture_context.encoder.flush(device);
            piston_window::clear([0.11; 4], g);

            piston_window::image(
                texture,
                c.transform.trans(0.0, 0.0).scale(
                    1.0 * (width / initial_width),
                    1.0 * (height / initial_height),
                ),
                g,
            );

            let transform = c.transform.trans(10.0, height - 10.0);
            piston_window::text::Text::new_color([0.27, 0.5, 0.2, 1.0], 12)
                .draw(&log.as_str(), glyphs, &c.draw_state, transform, g)
                .unwrap();

            // Update glyphs before rendering.
            glyphs.factory.encoder.flush(device);
        });

        Ok(())
    }

    // pub fn keyboard_event(&mut self) {
    //     let window = &mut self.window;

    //     match self.events.next(window) {
    //         Some(e) => {
    //             if let Some(input::Button::Keyboard(key)) = e.press_args() {
    //                 if key == input::Key::L {
    //                     state.lamp_enabled = !state.lamp_enabled;
    //                 }
    //                 if key == input::Key::Up {
    //                     state.forward = true;
    //                 }
    //                 if key == input::Key::Down {
    //                     state.backward = true;
    //                 }
    //                 if key == input::Key::Right {
    //                     state.right = true;
    //                 }
    //                 if key == input::Key::Left {
    //                     state.left = true;
    //                 }
    //                 info!("Pressed keyboard key '{:?}'", key);
    //             }
    //             if let Some(input::Button::Keyboard(key)) = e.release_args() {
    //                 if key == input::Key::Up {
    //                     state.forward = false;
    //                 }
    //                 if key == input::Key::Down {
    //                     state.backward = false;
    //                 }
    //                 if key == input::Key::Right {
    //                     state.right = false;
    //                 }
    //                 if key == input::Key::Left {
    //                     state.left = false;
    //                 }
    //                 info!("Released keyboard key '{:?}'", key);
    //             }
    //             if let Some(_) = e.close_args() {
    //                 debug!("Exiting...");
    //                 break;
    //             }
    //             // app.update(state);
    //         }
    //         None => {}
    //     }
    // }
}
