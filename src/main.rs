use std::fs::File;
use std::path::Path;
use std::env;

#[macro_use]
extern crate log;

mod error;
mod image;
mod codecs {
    pub mod png;
    pub mod ppm;
}

use crate::codecs::png::*;
use crate::codecs::ppm::*;
use crate::image::WriteImage;
use crate::image::ReadImage;

fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .init();
    let args: Vec<String> = env::args().collect();
    
    let path = Path::new(&args[1]);
    let file = File::open(&path).unwrap();

    let image: Box<PngImage> = PngImage::read_image(file).unwrap();

    let out_path = Path::new("out.ppm");
    let out_file = File::create(&out_path).unwrap();

    PpmImage::write_image(out_file, image.as_ref()).unwrap();
}
