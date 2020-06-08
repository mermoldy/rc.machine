extern crate find_folder;
use druid::widget::{ImageData, SvgData};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

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
