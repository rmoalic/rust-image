use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct DecodingError {
    pub str: String,
}

impl DecodingError {
    fn new(s: &str) -> Self {
        DecodingError {
            str: String::from(s)
        }
    }
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Decoding Error: {}", self.str)
    }
}

impl std::error::Error for DecodingError {

}

#[derive(Debug)]
pub enum ImageError {
    IO(std::io::Error),
    Decoding(DecodingError),
}

impl From<std::io::Error> for ImageError {
    fn from(e: std::io::Error) -> Self {
        ImageError::IO(e)
    }
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for ImageError {
    fn from(e: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        ImageError::Decoding(DecodingError {
            str: format!("mom error: {}", e)
        })
    }
}
