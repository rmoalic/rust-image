use bitstream_io::{BitReader, BitRead, BitWriter, BitWrite, LittleEndian};
use std::convert::TryInto;
use crate::hashs::adler32::Adler32;
use crate::compress::lzss::{LzssCode, get_block_lenght_and_distance, lzss_decode};
use crate::compress::huffman::{generate_dynamic_deflate_tree};

#[derive(Debug)]
enum DeflateCompression {
    None,
    Fixed,
    Dynamic
}

enum DecodeError {
    
}

pub fn decode_zlib(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    assert!(data.len() >= 5);
    let len = data.len();
    let cmf: u8 = data[0];
    let flg: u8 = data[1];
    
    assert_eq!((cmf & 0b1111), 8); // CM is deflate
    assert!((cmf >> 4) <= 7); // Cinfo <= 7
    assert_eq!((((cmf as u16) << 8) & flg as u16) % 31, 0);
    assert_eq!(flg >> 6 & 0b1, 0); // Preset dictionary not supported;

    let lz_window: u32 = 0b1 << ((cmf >> 4) + 8);
    
    let compression_level: u8 = flg & 0b111;
    dbg!(compression_level);

    let compressed_data = &data[2..len - 4];

    let adler: u32 = u32::from_be_bytes(data[len - 4 ..].try_into().unwrap());
    /*let mut hash = Adler32::new();
    hash.update(&compressed_data);
    let calculated_hash = hash.finalise();
    assert_eq!(adler, calculated_hash);*/
    //TODO: check adler32

    let ret = decode(compressed_data)?;

    return Ok(ret);
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut bit_reader: BitReader<&[u8], LittleEndian> = BitReader::new(data);
    let mut ret: Vec<u8> = Vec::new();
    println!("{:?}", data);
    
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
                let mut data = vec!(0; len as usize);
                bit_reader.read_bytes(data.as_mut())?;
                ret.extend(data); //TODO: improve copy (double copy)
            },
            DeflateCompression::Fixed => {
                let decode_tree = crate::compress::huffman::generate_fixed_deflate_tree(); //TODO: cache Tree
                let distance_tree = crate::compress::huffman::generate_fixed_deflate_distance_tree(); //TODO: cache

                let mut raw: Vec<LzssCode> = Vec::new();

                while let Ok((_code_len, code)) = decode_tree.read_one(&mut bit_reader) {
                    if code <= 255 {
                        raw.push(LzssCode::Val{code: code as u8});
                        continue;
                    }
                    if code == 256 {
                        break;
                    }
                    let (lenght, distance): (u16, u16) = get_block_lenght_and_distance(code as u16, &distance_tree, &mut bit_reader)?;
                    raw.push(LzssCode::Block{lenght: lenght, distance: distance});
                }
                
                let decoded: Vec<u8> = lzss_decode(&raw);
                ret.extend(decoded);
            },
            DeflateCompression::Dynamic => {
                let (decode_tree, code_distance_tree) = generate_dynamic_deflate_tree(&mut bit_reader)?;

                let mut raw: Vec<LzssCode> = Vec::new();

                while let Ok((_code_len, code)) = decode_tree.read_one(&mut bit_reader) {
                    if code <= 255 {
                        raw.push(LzssCode::Val{code: code as u8});
                        continue;
                    }
                    if code == 256 {
                        break;
                    }
//                    let (_len, distance) = code_distance_tree.read_one(&mut bit_reader).unwrap();
                    let (lenght, distance): (u16, u16) = get_block_lenght_and_distance(code as u16, &code_distance_tree, &mut bit_reader)?;
                    raw.push(LzssCode::Block{lenght: lenght as u16, distance: distance as u16});
                }
                
                let decoded: Vec<u8> = lzss_decode(&raw);
                ret.extend(decoded);

            }
        }

        if bfinal { break; }
    }

    let mut last = bit_reader.read_bit();
    while ! last.is_err() {
        println!("Error not empty: {}", last.unwrap());
        last = bit_reader.read_bit();
    }
    Ok(ret)
}

#[test]
fn test_decode_no_compress() {
    let code = vec!(120, 1, 1, 21, 0, 234, 255, 72, 101, 108, 108, 111, 32, 98, 108, 97, 104, 32, 98, 108, 97, 104, 32, 98, 108, 97, 104, 33, 81, 157, 7, 59);
    let res = decode_zlib(&code[..]).unwrap();
    assert_eq!(res, b"Hello blah blah blah!");
}

#[test]
fn test_decode_simple() {
    let code = vec!(120, 156, 243, 72, 205, 201, 201, 87, 40, 73, 45, 46, 81, 48, 52, 50, 6, 0, 37, 76, 4, 139);
    let res = decode_zlib(&code[..]).unwrap();
    assert_eq!(res, b"Hello test 123");
}

#[test]
fn test_decode_repeating() {
    let code = vec!(120, 156, 243, 72, 205, 201, 201, 87, 72, 202, 73, 204, 64, 16, 138, 0, 81, 157, 7, 59);
    let res = decode_zlib(&code[..]).unwrap();
    assert_eq!(res, b"Hello blah blah blah!");
}

#[test]
fn test_decode_repeating2() {
    let code = vec!(120, 156, 243, 72, 205, 201, 201, 87, 72, 202, 73, 204, 64, 16, 138, 10, 41, 249, 5, 48, 92, 2, 0, 205, 203, 11, 216);
    let res = decode_zlib(&code[..]).unwrap();

}

#[test]
fn test_decode_repeating3() {
    let message = b"Hello blah blah blah! Hello blah! dop dop";
    let code = vec!(120, 156, 243, 72, 205, 201, 201, 87, 72, 202, 73, 204, 64, 16, 138, 10, 30, 112, 81, 69, 133, 148, 252, 2, 16, 6, 0, 38, 85, 13, 237);
    let res = decode_zlib(&code[..]).unwrap();
    println!("expected: {}\ndecoded:  {}", String::from_utf8_lossy(message), String::from_utf8_lossy(&res));
    assert_eq!(res, message);
}

#[test]
fn test_decode_repeating4() {
    let message = b"Hello blah blah blah! Hello blah blah! dop dop";
    let code = vec!(120, 156, 243, 72, 205, 201, 201, 87, 72, 202, 73, 204, 64, 16, 138, 10, 30, 168, 162, 138, 10, 41, 249, 5, 32, 12, 0, 113, 120, 15, 164);
    let res = decode_zlib(&code[..]).unwrap();
    println!("expected: {}\ndecoded:  {}", String::from_utf8_lossy(message), String::from_utf8_lossy(&res));
    assert_eq!(res, message);
}

#[test]
fn test_decode_repeating5() {
    let message = b"Hello blah blah blah! Hello blah blah blah blah! dop dop";
    let code = vec!(120, 156, 243, 72, 205, 201, 201, 87, 72, 202, 73, 204, 64, 16, 138, 10, 30, 88, 68, 161, 82, 41, 249, 5, 32, 12, 0, 33, 134, 19, 18);
    let res = decode_zlib(&code[..]).unwrap();
    println!("expected: {}\ndecoded:  {}", String::from_utf8_lossy(message), String::from_utf8_lossy(&res));
    assert_eq!(res, message);
}


#[test]
fn test_decode_lorem() {
    let code = vec!(120, 156, 77, 81, 75, 78, 229, 64, 12, 188, 138, 15, 16, 229, 20, 35, 86, 192, 192, 98, 216, 155, 110, 19, 44, 245, 39, 207, 109, 71, 28, 159, 234, 188, 199, 240, 22, 145, 28, 187, 186, 92, 85, 126, 236, 38, 149, 116, 31, 81, 41, 247, 210, 141, 134, 58, 113, 21, 95, 40, 245, 54, 36, 185, 120, 24, 113, 214, 93, 71, 210, 182, 145, 20, 245, 149, 222, 244, 224, 26, 131, 62, 56, 105, 209, 161, 131, 220, 116, 184, 94, 66, 104, 47, 156, 196, 24, 168, 135, 24, 73, 232, 18, 24, 163, 7, 56, 55, 151, 149, 158, 163, 20, 174, 84, 192, 142, 94, 207, 218, 23, 114, 169, 59, 214, 135, 83, 155, 83, 154, 10, 240, 184, 8, 104, 39, 105, 97, 72, 149, 219, 244, 70, 241, 127, 251, 252, 111, 137, 120, 106, 206, 90, 165, 57, 252, 20, 125, 23, 235, 43, 253, 233, 77, 18, 29, 81, 246, 112, 118, 249, 225, 15, 26, 188, 169, 187, 222, 153, 248, 1, 67, 69, 229, 173, 1, 181, 97, 101, 57, 67, 58, 227, 184, 4, 251, 111, 181, 210, 107, 92, 213, 93, 45, 158, 184, 155, 208, 229, 46, 144, 15, 100, 6, 239, 11, 138, 216, 20, 12, 211, 242, 74, 47, 159, 60, 164, 20, 68, 48, 36, 211, 102, 124, 104, 102, 170, 112, 243, 215, 146, 210, 193, 166, 152, 53, 246, 126, 166, 42, 168, 244, 29, 29, 185, 170, 3, 101, 198, 183, 179, 225, 66, 10, 207, 84, 59, 226, 29, 11, 158, 32, 185, 121, 54, 211, 172, 41, 230, 6, 220, 106, 165, 127, 78, 103, 242, 169, 219, 46, 134, 8, 116, 212, 158, 239, 173, 42, 8, 44, 71, 93, 233, 137, 65, 138, 119, 252, 165, 243, 206, 135, 58, 203, 45, 210, 25, 143, 133, 27, 96, 223, 193, 160, 213, 41);
    let res = decode_zlib(&code[..]).unwrap();
    let s = String::from_utf8_lossy(&res);
    println!("decoded: {}", s);
    assert_eq!(res, b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus facilisis tristique placerat. Fusce quis lacus ante. Nullam lectus odio, tempor ut nulla et, scelerisque laoreet nulla. Nulla facilisi. Nunc a condimentum libero. Donec vulputate nulla eu sagittis facilisis. Donec ut magna eget lorem consequat consequat. Quisque quis lorem laoreet, tristique felis a, feugiat odio. Phasellus sed gravida mi. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Ut ullamcorper euismod magna eget interdum. Mauris maximus vitae libero ut rutrum. ");
}

#[test]
fn test_decode_lorem1() {
    let code = vec!(120, 1, 77, 81, 73, 78, 196, 48, 16, 252, 74, 61, 96, 148, 87, 32, 78, 172, 7, 184, 55, 118, 79, 104, 201, 75, 198, 110, 71, 60, 159, 114, 24, 6, 14, 145, 156, 94, 170, 107, 121, 168, 77, 51, 108, 235, 35, 35, 214, 84, 27, 186, 57, 36, 171, 159, 16, 106, 233, 26, 92, 125, 52, 72, 180, 205, 122, 176, 178, 66, 147, 249, 130, 119, 219, 37, 143, 142, 179, 4, 75, 214, 173, 195, 155, 117, 183, 203, 80, 108, 73, 130, 54, 225, 212, 253, 232, 65, 113, 25, 108, 179, 198, 113, 41, 174, 11, 158, 70, 74, 146, 145, 136, 206, 90, 141, 86, 79, 112, 205, 27, 207, 15, 71, 153, 93, 76, 6, 92, 78, 74, 216, 9, 154, 132, 84, 245, 218, 189, 66, 220, 174, 207, 255, 18, 32, 147, 115, 180, 172, 197, 169, 39, 217, 135, 182, 186, 224, 174, 22, 13, 216, 71, 218, 134, 139, 235, 47, 254, 64, 151, 213, 220, 201, 237, 38, 226, 119, 152, 44, 178, 172, 133, 44, 86, 158, 164, 45, 52, 233, 176, 227, 50, 196, 255, 94, 11, 94, 41, 109, 178, 251, 145, 120, 204, 93, 137, 82, 209, 205, 144, 51, 61, 163, 246, 19, 206, 58, 86, 35, 194, 148, 188, 224, 229, 83, 186, 166, 68, 11, 186, 70, 172, 77, 118, 139, 130, 108, 11, 158, 91, 48, 236, 210, 140, 189, 34, 94, 15, 87, 149, 47, 251, 96, 133, 148, 38, 59, 66, 70, 126, 155, 52, 38, 100, 212, 140, 92, 105, 111, 63, 113, 133, 206, 205, 216, 154, 69, 11, 99, 94, 96, 86, 11, 222, 28, 135, 243, 161, 182, 77, 27, 148, 228, 115, 141, 255, 165, 26, 1, 90, 28, 121, 193, 163, 16, 148, 123, 242, 101, 51, 231, 221, 92, 24, 194, 97, 233, 12, 169, 13, 111, 28, 251, 6, 193, 160, 213, 41);
    let res = decode_zlib(&code[..]).unwrap();
    let s = String::from_utf8_lossy(&res);
    println!("decoded: {}", s);
    assert_eq!(res, b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus facilisis tristique placerat. Fusce quis lacus ante. Nullam lectus odio, tempor ut nulla et, scelerisque laoreet nulla. Nulla facilisi. Nunc a condimentum libero. Donec vulputate nulla eu sagittis facilisis. Donec ut magna eget lorem consequat consequat. Quisque quis lorem laoreet, tristique felis a, feugiat odio. Phasellus sed gravida mi. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Ut ullamcorper euismod magna eget interdum. Mauris maximus vitae libero ut rutrum. ");
}

#[test]
#[ignore]
fn test_decode_lorem9() {
    let code = vec!(120, 218, 77, 81, 75, 78, 229, 64, 12, 188, 138, 15, 16, 229, 20, 35, 86, 192, 192, 98, 216, 155, 110, 19, 44, 245, 39, 207, 109, 71, 28, 159, 234, 188, 199, 240, 22, 145, 28, 187, 186, 92, 85, 126, 236, 38, 149, 116, 31, 81, 41, 247, 210, 141, 134, 58, 113, 21, 95, 40, 245, 54, 36, 185, 120, 24, 113, 214, 93, 71, 210, 182, 145, 20, 245, 149, 222, 244, 224, 26, 131, 62, 56, 105, 209, 161, 131, 220, 116, 184, 94, 66, 104, 47, 156, 196, 24, 168, 135, 24, 73, 232, 18, 24, 163, 7, 56, 55, 151, 149, 158, 163, 20, 174, 84, 192, 142, 94, 207, 218, 23, 114, 169, 59, 214, 135, 83, 155, 83, 154, 10, 240, 184, 8, 104, 39, 105, 97, 72, 149, 219, 244, 70, 241, 127, 251, 252, 111, 137, 120, 106, 206, 90, 165, 57, 252, 20, 125, 23, 235, 43, 253, 233, 77, 18, 29, 81, 246, 112, 118, 249, 225, 15, 26, 188, 169, 187, 222, 153, 248, 1, 67, 69, 229, 173, 1, 181, 97, 101, 57, 67, 58, 227, 184, 4, 251, 111, 181, 210, 107, 92, 213, 93, 45, 158, 184, 155, 208, 229, 46, 144, 15, 100, 6, 239, 11, 138, 216, 20, 12, 211, 242, 74, 47, 159, 60, 164, 20, 68, 48, 36, 211, 102, 124, 104, 102, 170, 112, 243, 215, 146, 210, 193, 166, 152, 53, 246, 126, 166, 42, 168, 244, 29, 29, 185, 170, 3, 101, 198, 183, 179, 225, 66, 10, 207, 84, 59, 226, 29, 11, 158, 32, 185, 121, 54, 211, 172, 41, 230, 6, 220, 106, 165, 127, 78, 103, 242, 169, 219, 46, 134, 8, 116, 212, 158, 239, 173, 42, 8, 44, 71, 93, 233, 137, 65, 138, 119, 252, 165, 243, 206, 135, 58, 203, 45, 210, 25, 143, 133, 27, 96, 223, 193, 160, 213, 41);
    let res = decode_zlib(&code[..]).unwrap();
    let s = String::from_utf8_lossy(&res);
    println!("decoded: {}", s);
    assert_eq!(res, b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus facilisis tristique placerat. Fusce quis lacus ante. Nullam lectus odio, tempor ut nulla et, scelerisque laoreet nulla. Nulla facilisi. Nunc a condimentum libero. Donec vulputate nulla eu sagittis facilisis. Donec ut magna eget lorem consequat consequat. Quisque quis lorem laoreet, tristique felis a, feugiat odio. Phasellus sed gravida mi. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Ut ullamcorper euismod magna eget interdum. Mauris maximus vitae libero ut rutrum. ");
}
