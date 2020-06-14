extern crate find_folder;
use druid::widget::{ImageData, SvgData};
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, Instant};

pub fn load_assets_folder(path: &str) -> Result<PathBuf, io::Error> {
    match find_folder::Search::ParentsThenKids(1, 4).for_folder(format!("assets/{}", path).as_str())
    {
        Ok(res) => Ok(res),
        Err(err) => {
            return Err(io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to load directory: {}", err),
            ))
        }
    }
}

pub fn load_image(name: &str) -> Result<ImageData, io::Error> {
    let dir = load_assets_folder("images")?;

    match ImageData::from_file(dir.join(name)) {
        Ok(res) => Ok(res),
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to image {}: {}", name, err),
            ))
        }
    }
}

pub fn load_svg(name: &str) -> Result<SvgData, io::Error> {
    let dir = load_assets_folder("svg")?;
    let svg_str = fs::read_to_string(dir.join(name)).expect("Unable to read file");
    match SvgData::from_str(svg_str.as_str()) {
        Ok(svg) => Ok(svg),
        Err(err) => {
            error!("{}", err);
            error!("Using an empty SVG instead.");
            Ok(SvgData::default())
        }
    }
}

pub struct FPSCounter {
    frames: VecDeque<Instant>,
}

impl FPSCounter {
    pub fn new(limit: u8) -> FPSCounter {
        FPSCounter {
            frames: VecDeque::with_capacity(limit as usize),
        }
    }

    pub fn tick(&mut self) -> u8 {
        let now = Instant::now();
        let second_ago = now - Duration::from_secs(1);

        while self.frames.front().map_or(false, |t| *t < second_ago) {
            self.frames.pop_front();
        }

        self.frames.push_back(now);
        self.frames.len() as u8
    }
}
