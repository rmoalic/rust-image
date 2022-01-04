extern crate nom;
extern crate miniz_oxide;

use nom::bytes::complete::*;
use nom::error::Error;
use nom::number::complete::*;
use nom::sequence::{tuple, terminated};
use nom::multi::count;
use nom::IResult;
use std::str;
use miniz_oxide::inflate::decompress_to_vec_zlib;
use std::ops::Div;
use std::io::Read;

use crate::image::{GenericImageTo, GenericImage, GenericImageColors, ReadImage};
use crate::error::*;

#[derive(Debug)]
struct Chunk<'a> {
    len: u32,
    name: &'a str,
    data: &'a [u8],
    crc: u32,
}

impl Chunk<'_> {
    fn check_crc(&self) -> bool {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(self.name.as_bytes());
        hasher.update(self.data);
        let computed_crc: u32 = hasher.finalize();
        return computed_crc == self.crc;
    }
}

#[derive(Debug)]
struct IHDR {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: ColorType,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

#[derive(Debug)]
struct IDAT<'a> {
    data: &'a [u8]
}


#[derive(Debug)]
pub struct PngImage {
    ihdr: Option<IHDR>,
    idat: Vec<u8>,
    bpp: u32,
    color_index: Option<Vec<(u8, u8, u8)>>,
    has_end: bool,
}

#[derive(Debug, PartialEq, PartialOrd)]
#[repr(u8)]
enum FilterType {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4
}

impl FilterType {
    fn from_u8(val: u8) -> Self {
        match val {
            0 => FilterType::None,
            1 => FilterType::Sub,
            2 => FilterType::Up,
            3 => FilterType::Average,
            4 => FilterType::Paeth,
            _ => panic!("unknown type")
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
#[repr(u8)]
enum ColorType {
    GrayScale = 0,
    TrueColor = 2,
    IndexedColor = 3,
    GrayScaleAlpha = 4,
    TrueColorAlpha = 6
}

impl ColorType  {
    fn from_u8(val: u8) -> Self {
        match val {
            0 => ColorType::GrayScale,
            2 => ColorType::TrueColor,
            3 => ColorType::IndexedColor,
            4 => ColorType::GrayScaleAlpha,
            6 => ColorType::TrueColorAlpha,
            _ => panic!("unknown type")
        }
    }
}





fn peath_predictor(a: i16, b: i16, c: i16) -> u8 {
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    let ret;

    if pa <= pb && pa <= pc {
        ret = a;
    } else if pb <= pc {
        ret = b;
    } else {
        ret = c;
    }

    return (ret % 255) as u8
}


fn map_indexed_color(index: &Vec<(u8, u8, u8)>, img: &Vec<u8>, size: usize) -> Vec<u8> {
    let mut ret: Vec<u8> = Vec::with_capacity(size);
    
    for b in img {
        let (r, g, b) = index.get(*b as usize).unwrap();
        ret.push(*r);
        ret.push(*g);
        ret.push(*b);
    }
    return ret;
}



impl PngImage {
    fn new() -> PngImage {
        PngImage {
            ihdr: None,
            idat: Vec::new(),
            bpp: 4,
            color_index: None,
            has_end: false,
        }
    }

    fn filter_scanline(&self, prev: &[u8], sl: &mut [u8], filter_method: FilterType) {
        let bpp = self.bpp as usize;
        let scanline_len = self.scanline_len();

        match filter_method {
            FilterType::None => {

            }
            FilterType::Sub => {
                for i in bpp..scanline_len {
                    sl[i] = sl[i].overflowing_add(sl[i-bpp]).0;
                }
            }
            FilterType::Up => {
                for i in 0..scanline_len {
                    sl[i] = sl[i].overflowing_add(prev[i]).0;
                }
            }
            FilterType::Average => {
                for i in 0..bpp {
                    sl[i] = sl[i].overflowing_add(prev[i].div(2)).0;
                }

                for i in bpp..scanline_len {
                    sl[i] = sl[i].overflowing_add((sl[i-bpp] as u16 + prev[i] as u16).div(2) as u8).0;
                }
            }
            FilterType::Paeth => {
                for i in 0..bpp {
                    sl[i] = sl[i].overflowing_add(peath_predictor(0, prev[i] as i16, 0)).0;
                }

                for i in bpp..scanline_len {
                    sl[i] = sl[i].overflowing_add(peath_predictor(sl[i - bpp] as i16, prev[i] as i16, prev[i - bpp] as i16)).0;
                }
            }
        }
    }

    fn nb_pixels(&self) -> u32 {
        let ihdr = self.ihdr.as_ref().unwrap();

        return ihdr.height * ihdr.width;
    }

    fn scanline_nb_pixel_components(&self) -> u32 {
        match self.ihdr.as_ref().unwrap().color_type {
            ColorType::GrayScale => 1,
            ColorType::GrayScaleAlpha => 2,
            ColorType::TrueColor => 3,
            ColorType::TrueColorAlpha => 4,
            ColorType::IndexedColor => 1,
        }
    }

    fn scanline_pixel_data_size(&self) -> usize {
        self.nb_pixels() as usize * self.scanline_nb_pixel_components() as usize
    }

    fn scanline_len(&self) -> usize {
        return self.ihdr.as_ref().unwrap().width as usize * self.scanline_nb_pixel_components() as usize;
    }
    
    fn decode_scanlines(&self) -> Vec<u8> {
        let mut decoded = decompress_to_vec_zlib(self.idat.as_slice()).unwrap();
        let scanline_len = self.scanline_len() as usize;


        let mut ret: Vec<u8> = Vec::with_capacity(self.scanline_pixel_data_size());
        let sl0: Vec<u8> = vec!(0u8; self.scanline_pixel_data_size());

        let mut prev_scanline: &[u8] = sl0.as_ref();

        for scanline in decoded.chunks_mut(scanline_len + 1) {
            let filter_method = FilterType::from_u8(scanline[0]);
            let sl = scanline[1..].as_mut();
            
            debug!("method: {:?}", filter_method);
            
            self.filter_scanline(prev_scanline, sl, filter_method);
            ret.extend_from_slice(sl);
            prev_scanline = sl;
        }

        return ret;
    }

    fn decode_to_rgb(&self) -> Vec<u8> {
        let img = self.decode_scanlines();
        let mut ret: Vec<u8>;

        match self.ihdr.as_ref().unwrap().color_type {
            ColorType::IndexedColor => {
                ret = map_indexed_color(self.color_index.as_ref().unwrap(), &img, (self.nb_pixels() * 3) as usize);
            }
            ColorType::TrueColorAlpha => {
                ret = Vec::with_capacity((self.nb_pixels() * 3) as usize);
//                ret = vec!(0u8; (self.nb_pixels() * 3) as usize);

                for (i, b) in img.chunks(4).enumerate() {
  //                  debug!("{}: {:?}", i ,b );
                    ret.push(b[0]);
                    ret.push(b[1]);
                    ret.push(b[2]);
                    
//                    ret[i..i+3].copy_from_slice(&b[0..3]);
                }
            }
            ColorType::TrueColor => {
                ret = img;
            }
            _ => todo!("Colortype")
        }
        return ret;
    }
}

impl GenericImageTo for PngImage {
    fn to_rgb(&self) -> Result<GenericImage, ImageError> {
        let ihdr = self.ihdr.as_ref().unwrap();
        let data = self.decode_to_rgb();
        let ret: GenericImage = GenericImage {
            data,
            colors: GenericImageColors::RGB,
            height: ihdr.height,
            width: ihdr.width,
        };
        return Ok(ret);
    }
    
    fn to_rgba(&self) -> Result<GenericImage, ImageError> {
        todo!();
    } 
    fn to_g(&self) -> Result<GenericImage, ImageError> {
        todo!();
    }
}

impl<R: Read> ReadImage<R> for PngImage {
    fn read_image(mut reader: R) -> Result<Box<Self>, ImageError> {
        let mut data: Vec<u8> = Vec::new();
        reader.read_to_end(&mut data)?;

        let image: PngImage = parse_png(data.as_ref())?.1;
        return Ok(Box::new(image));
    }
}

fn parse_idat(idat_chunk: Chunk) -> Result<IDAT, nom::Err<Error<&[u8]>>> {
    assert_eq!(idat_chunk.name, "IDAT");
    
    let idat = IDAT {
        data: idat_chunk.data,
    };

    return Ok(idat);
}

fn parse_iend(iend_chunk: Chunk) -> Result<bool, nom::Err<Error<&[u8]>>> {
    assert_eq!(iend_chunk.name, "IEND");
    assert_eq!(iend_chunk.len, 0);
    return Ok(true);
}

fn parse_ihdr(ihdr_chunk: Chunk) -> Result<IHDR, nom::Err<Error<&[u8]>>> {
    assert_eq!(ihdr_chunk.name, "IHDR");
    assert_eq!(ihdr_chunk.len, 13);

    let (
        _i,(
        width,
        height,
        bit_depth,
        color_type,
        compression_method,
        filter_method,
        interlace_method)
    ) = tuple((be_u32, be_u32, u8, u8, u8, u8, u8))(ihdr_chunk.data)?;

    let ihdr = IHDR {
        width,
        height,
        bit_depth,
        color_type: ColorType::from_u8(color_type),
        compression_method,
        filter_method,
        interlace_method,
    };

    assert!([1, 2, 4, 8, 16].contains(&ihdr.bit_depth));
    assert!([0, 2, 3, 4, 6].contains(&color_type));
    assert!([0, 1].contains(&ihdr.interlace_method));
    assert_eq!(ihdr.compression_method, 0);
    assert_eq!(ihdr.filter_method, 0);
    return Ok(ihdr);
}

fn parse_text(text_chunk: Chunk) -> Result<&str, nom::Err<Error<&[u8]>>> {
    assert_eq!(text_chunk.name, "tEXt");

    let text = str::from_utf8(text_chunk.data).unwrap();
    info!("text: {}", text);

    return Ok(&text);
}

fn parse_ztxt(text_chunk: Chunk) -> Result<(&str, String), nom::Err<Error<&[u8]>>> {
    assert_eq!(text_chunk.name, "zTXt");
    let (r, keyword) = terminated(take_while(|b: u8| b != 0), tag([0x0]))(text_chunk.data)?;
    let (r, compression_method) = u8(r)?;

    assert_eq!(compression_method, 0);
    let keyword_utf = str::from_utf8(keyword).unwrap();

    let decoded = decompress_to_vec_zlib(r).unwrap();
    let text = String::from_utf8(decoded).unwrap();
    info!("ztxt {}: {}", keyword_utf, text);

    return Ok((&keyword_utf, text));
}

fn parse_phys(chunk: Chunk) -> Result<(u32, u32, bool), nom::Err<Error<&[u8]>>> {
    assert_eq!(chunk.name, "pHYs");
    assert_eq!(chunk.len, 9);
    let (r, ppux) = be_u32(chunk.data)?;
    let (r, ppuy) = be_u32(r)?;
    let (_r, unit) = be_u8(r)?;

    if unit == 0 {
        info!("phys: ppuX: {} ppuY: {}", ppux, ppuy);
    } else if unit == 1 {
        info!("phys: ppuX: {}m ppuY: {}m", ppux, ppuy);
    } else {
        panic!("unknown unit");
    }

    Ok((ppux, ppuy, unit == 1))
}

fn parse_time(chunk: Chunk) -> Result<u8, nom::Err<Error<&[u8]>>> {
    assert_eq!(chunk.name, "tIME");
    assert_eq!(chunk.len, 7);

    let (_, (year, month, day, hour, minute, second)) = tuple((be_u16, u8, u8, u8, u8, u8))(chunk.data)?;

    info!("time: {}-{:#02}-{:#02} {:#02}:{:#02}:{:#02}", year, month, day, hour, minute, second);

    Ok(0)
}

fn parse_plte(chunk: Chunk) -> Result<Vec<(u8, u8, u8)>, nom::Err<Error<&[u8]>>> {
    assert_eq!(chunk.name, "PLTE");
    assert_eq!(chunk.len % 3, 0);
    let nb_colors = chunk.len / 3;

    let (_, colors) = count(tuple((u8, u8, u8)), nb_colors as usize)(chunk.data)?;

    info!("plte: {:?}", colors);

    Ok(colors)
}

fn parse_chunk(chunk: &[u8]) -> IResult<&[u8], Chunk> {
    let (r, len): (&[u8], u32) = be_u32(chunk)?;
    let (r, name_bytes): (&[u8], &[u8]) = take(4 as u32)(r)?;
    let (r, data): (&[u8], &[u8]) = take(len)(r)?;
    let (r, crc): (&[u8], u32) = be_u32(r)?;

    let name = str::from_utf8(name_bytes).unwrap();

    info!("\tChunk name: {}, size: {}, crc: {}", name, len, crc);

    let chunk = Chunk { len, name, data, crc };

    if ! chunk.check_crc() {
        panic!("Chunk crc error");
    }

    Ok((r, chunk))
}

pub fn parse_png(chunk: &[u8]) -> IResult<&[u8], PngImage> {
    debug!("Parsing png");
    let (r, _) = tag([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])(chunk)?;
    let mut suite = r;
    let mut image = PngImage::new();

    while suite.len() > 0 {
        let p = parse_chunk(suite)?;
        match p.1.name {
            "IHDR" => {
                image.ihdr = Some(parse_ihdr(p.1).unwrap());
                info!("IHDR: {:?}", image.ihdr);
            }
            "IDAT" => {
                image.idat.extend(parse_idat(p.1).unwrap().data);
                info!("IDAT: new chunk added");
            }
            "IEND" => {
                image.has_end = true;
                info!("IEND: {}", parse_iend(p.1).unwrap());
            }
            "tEXt" => {
                parse_text(p.1).unwrap();
            }
            "zTXt" => {
                parse_ztxt(p.1).unwrap();
            }
            "pHYs" => {
                parse_phys(p.1).unwrap();
            }
            "tIME" => {
                parse_time(p.1).unwrap();
            }
            "PLTE" => {
                image.color_index = Some(parse_plte(p.1).unwrap());
            }
            name => warn!("no parsing for chunk: {}", name)
        }

        suite = p.0;
    }
    debug!("End of parsing");
    Ok((r, image))
}
