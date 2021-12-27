extern crate nom;
extern crate inflate;

use nom::bytes::complete::*;
use nom::error::Error;
use nom::number::complete::*;
use nom::sequence::tuple;
use nom::IResult;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;
use std::io::Write;
use inflate::inflate_bytes_zlib;

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
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

#[derive(Debug)]
struct IDAT<'a> {
    data: &'a [u8]
}


#[derive(Debug)]
struct PngImage {
    ihdr: Option<IHDR>,
    idat: Vec<u8>,
    has_end: bool,
}

impl PngImage {
    fn new() -> PngImage {
        PngImage {
            ihdr: None,
            idat: Vec::new(),
            has_end: false,
        }
    }

    fn to_ppn(self, w: &mut dyn Write) {
        let ihdr = self.ihdr.unwrap();

        write!(w, "P3\n").unwrap();
        write!(w, "{} {}\n255\n", ihdr.width, ihdr.height).unwrap();


        let decoded = inflate_bytes_zlib(self.idat.as_ref()).unwrap();
        assert_eq!(decoded.len() % 4, 0);
        for b in decoded.chunks(4) {
            write!(w, "{} {} {}\n", b[0], b[1], b[2]).unwrap();                
        }
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
        color_type,
        compression_method,
        filter_method,
        interlace_method,
    };

    assert!([1, 2, 4, 8, 16].contains(&ihdr.bit_depth));
    assert!([0, 2, 3, 4, 6].contains(&ihdr.color_type));
    assert!([0, 1].contains(&ihdr.interlace_method));
    assert_eq!(ihdr.compression_method, 0);
    assert_eq!(ihdr.filter_method, 0);
    return Ok(ihdr);
}

fn parse_chunk(chunk: &[u8]) -> IResult<&[u8], Chunk> {
    let (r, len): (&[u8], u32) = be_u32(chunk)?;
    let (r, name_bytes): (&[u8], &[u8]) = take(4 as u32)(r)?;
    let (r, data): (&[u8], &[u8]) = take(len)(r)?;
    let (r, crc): (&[u8], u32) = be_u32(r)?;

    let name = str::from_utf8(name_bytes).unwrap();

    println!("Chunk name: {}, size: {}, crc: {}", name, len, crc);

    let chunk = Chunk { len, name, data, crc };

    if ! chunk.check_crc() {
        panic!("Chunk crc error");
    }

    return Ok((r, chunk));
}

fn parse_png(chunk: &[u8]) -> IResult<&[u8], PngImage> {
    let (r, _) = tag([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])(chunk)?;
    let mut suite = r;
    let mut image = PngImage::new();
    
    while suite.len() > 0 {
        let p = parse_chunk(suite)?;
        match p.1.name {
            "IHDR" => {
                image.ihdr = Some(parse_ihdr(p.1).unwrap());
                println!("IHDR: {:?}", image.ihdr);
            }
            "IDAT" => {
                image.idat.extend(parse_idat(p.1).unwrap().data);
                println!("IDAT: new chunk added");
            }
            "IEND" => {
                image.has_end = true;
                println!("IEND: {}", parse_iend(p.1).unwrap());
            }
            name => println!("no parsing for chunk: {}", name)
        }

        suite = p.0;
    }

    return Ok((r, image));
}


fn write_ppn(image: PngImage) {
    let path = Path::new("test.ppn");
    let mut file = File::create(&path).unwrap();


    //let mut d: Vec<u8> = Vec::new();
    image.to_ppn(&mut file);
    
//    println!("ppn: {:?}", d);    
}

fn main() {
    let path = Path::new("test.png");
    let mut file = File::open(&path).unwrap();
    let mut data: Vec<u8> = Vec::new();

    match file.read_to_end(&mut data) {
        Err(why) => panic!("canor read file: {}", why),
        Ok(_) => println!("OK"),
    }

    let image: PngImage = parse_png(data.as_ref()).unwrap().1;

    write_ppn(image);
//    println!("Image {:?}", image);
}
