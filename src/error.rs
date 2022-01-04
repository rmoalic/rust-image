use std::fmt;
use std::fmt::Display;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use miniz_oxide::inflate::TINFLStatus;

#[derive(Debug)]
pub struct DecodingError {
    pub str: String,
}

impl DecodingError {
    pub fn new(s: &str) -> Self {
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
    Compression(TINFLStatus),
}

impl From<std::io::Error> for ImageError {
    fn from(e: std::io::Error) -> Self {
        ImageError::IO(e)
    }
}

impl From<TINFLStatus> for ImageError {
    fn from(e: TINFLStatus) -> Self {
        ImageError::Compression(e)
    }
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for ImageError {
    fn from(e: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        ImageError::Decoding(DecodingError {
            str: format!("mom error: {}", e)
        })
    }
}

impl From<Utf8Error> for ImageError {
    fn from(e: Utf8Error) -> Self {
        ImageError::Decoding(DecodingError {
            str: format!("utf8 error: {}", e)
        })
    }
}

impl From<FromUtf8Error> for ImageError {
    fn from(e: FromUtf8Error) -> Self {
        ImageError::Decoding(DecodingError {
            str: format!("from utf8 error: {}", e)
        })
    }
}
