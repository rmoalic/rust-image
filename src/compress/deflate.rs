extern crate nom;

/*
use nom::bytes::complete::*;
use nom::number::complete::*;
use nom::sequence::{tuple, terminated};
use nom::multi::count;
 */
use bitstream_io::{BitReader, BitRead, BitWriter, BitWrite, LittleEndian};

#[derive(Debug)]
enum DeflateCompression {
    None,
    Fixed,
    Dynamic
}

enum DecodeError {
    
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut bit_reader: BitReader<&[u8], LittleEndian> = BitReader::new(data);
    let mut ret: Vec<u8> = Vec::new();
    println!("{:?}", data);


    let cmf: u8 = bit_reader.read(8)?;
    let flg: u8 = bit_reader.read(8)?;

    assert_eq!((cmf & 0b1111), 8); // CM is deflate
    assert!((cmf >> 4) <= 7); // Cinfo <= 7
    assert_eq!((((cmf as u16) << 8) & flg as u16) % 31, 0);
    assert_eq!(flg >> 6 & 0b1, 0); // Preset dictionary not supported;

    let lz_window: u32 = 0b1 << ((cmf >> 4) + 8);
    
    let compression_level: u8 = flg & 0b111;
    dbg!(compression_level);

    let fixed_tree = crate::compress::huffman::generate_fixed_deflate_tree(); //TODO: cache Tree;
    
    loop {
        let block_header: u8 = bit_reader.read(3)?;
        let bfinal: bool = block_header & 0b1 == 1;
        let btype = match block_header >> 1 {
            0 => DeflateCompression::None,
            1 => DeflateCompression::Fixed,
            2 => DeflateCompression::Dynamic,
            _ => unreachable!()
        };

        println!("bfinal: {} btype: {:?}", &bfinal, &btype);

        match btype {
            DeflateCompression::None => {
                bit_reader.byte_align();
                let len: u16 = bit_reader.read(16)?;
                let nlen: u16 = bit_reader.read(16)?;
                assert_eq!(len , !nlen);
                let mut data = Vec::<u8>::with_capacity(len as usize);
                bit_reader.read_bytes(data.as_mut())?;
                ret.extend(data); //TODO: improve copy (double copy)
            },
            DeflateCompression::Fixed => {
                let mut arr: Vec<u8> = Vec::new();
                let mut writer = BitWriter::endian(&mut arr, LittleEndian);
                let mut written: u32 = 0;

                while let Ok((code_len, code)) = fixed_tree.read_one(&mut bit_reader) {
                    println!("{}\t({}): {}", code, code as u8 as char, code_len);
                    if code <= 255 {
                        writer.write(code_len, code)?;
                        written += code_len;
                        continue;
                    }
                    if code == 256 {
                        break;
                    }
                    writer.flush()?;
                    let (bits, value) = writer.into_unwritten();

                    let (len, d): (u16, Vec<u8>) = crate::compress::lzss::lzss_decode(code as u16, &arr, bits as u16, &mut bit_reader)?;
                    dbg!(&arr);
                    println!("decode {}: {:?}", len, d);

                    writer = BitWriter::endian(&mut arr, LittleEndian);
                    writer.write(bits, value)?;
                    writer.write_bytes(&d)?;

                    written += len as u32;
                }
                writer.write(written % 8, 0u8)?;
                writer.into_writer();

                ret.extend(arr);
            },
            DeflateCompression::Dynamic => {
                unimplemented!();
            }
        }


        if bfinal { break; }
    }
    let adler: u32 = bit_reader.read(32)?;
    //TODO: check adler32
    let mut last = bit_reader.read_bit();
    while ! last.is_err() {
        println!("Error not empty: {}", last.unwrap());
        last = bit_reader.read_bit();
    }
    Ok(ret)
}

#[test]
fn test_decode_simple() {
    let code = vec!(120, 156, 243, 72, 205, 201, 201, 87, 40, 73, 45, 46, 81, 48, 52, 50, 6, 0, 37, 76, 4, 139);
    let res = decode(&code[..]).unwrap();
    assert_eq!(res, b"Hello test 123");
}

#[test]
fn test_decode_repeating() {
    let code = vec!(120, 156, 243, 72, 205, 201, 201, 87, 72, 202, 73, 204, 64, 16, 138, 0, 81, 157, 7, 59);
    let res = decode(&code[..]).unwrap();
    assert_eq!(res, b"Hello blah blah blah!");
}
