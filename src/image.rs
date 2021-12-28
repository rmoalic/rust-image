use std::io::Write;
use std::io::Read;
use crate::error::ImageError;

pub enum GenericImageColors {
    RGB,
    RGBA,
    G
}

pub struct GenericImage {
    pub width: u32,
    pub height: u32,
    pub colors: GenericImageColors,
    pub data: Vec<u8>,
}

pub trait GenericImageTo {
    fn to_rgb(&self) -> Result<GenericImage, ImageError>;
    fn to_rgba(&self) -> Result<GenericImage, ImageError>;
    fn to_g(&self) -> Result<GenericImage, ImageError>;
}

pub trait WriteImage<W: Write, I: GenericImageTo> {
    fn write_image(writer: W, image: &I) -> Result<(), ImageError>;
}

pub trait ReadImage<R: Read> {
    fn read_image(reader: R) -> Result<Box<Self>, ImageError>;
}
