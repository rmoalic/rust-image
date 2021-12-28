use crate::image::*;
use crate::error::ImageError;
use std::io::Write;
use std::io::BufWriter;

pub struct PpmImage {}

impl<W: Write, I: GenericImageTo> WriteImage<W, I> for PpmImage {
    
    fn write_image(writer: W, image: &I) -> Result<(), ImageError> {
        let mut buf = BufWriter::new(writer);
        let img = image.to_rgb()?;

        write!(buf, "P6\n")?;
        write!(buf, "{} {}\n255\n", img.width, img.height)?;

        buf.write_all(&img.data)?;
        Ok(())
    }
}
