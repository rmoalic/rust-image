
use std::io::Read;
use std::cmp;
use bitstream_io::{BitReader, BitRead, LittleEndian};
use crate::compress::huffman;
use std::fmt::Debug;

const LZSS_MAX_DISTANCE: u16 = 32_768;
const LZSS_MAX_LENGHT: u16 = 258;

pub enum LzssCode {
    Val {code: u8},
    Block {lenght: u16, distance: u16}
}

impl Debug for LzssCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            LzssCode::Val {code} => {
                write!(f, "{}", *code as char);
            },
            LzssCode::Block {lenght, distance} => {
                write!(f, "({}, {})", *lenght, *distance);
            }
        }
        Ok(())
     }
}

pub fn get_block_lenght_and_distance<R: Read>(curr: u16, compressed_data: &mut BitReader<R, LittleEndian>) -> Result<(u16, u16), std::io::Error> {
    let lenght: u16;
    let distance: u16;
    assert!(curr >= 257);
/*
      Extra               Extra               Extra
 Code Bits Length(s) Code Bits Lengths   Code Bits Length(s)
 ---- ---- ------     ---- ---- -------   ---- ---- -------
  257   0     3       267   1   15,16     277   4   67-82
  258   0     4       268   1   17,18     278   4   83-98
  259   0     5       269   2   19-22     279   4   99-114
  260   0     6       270   2   23-26     280   4  115-130
  261   0     7       271   2   27-30     281   5  131-162
  262   0     8       272   2   31-34     282   5  163-194
  263   0     9       273   3   35-42     283   5  195-226
  264   0    10       274   3   43-50     284   5  227-257
  265   1  11,12      275   3   51-58     285   0    258
  266   1  13,14      276   3   59-66
*/
    if curr <= 264 {
        lenght = 3 + (curr - 257);
    } else if curr <= 268 {
        let min_len: u16 = 11 + (curr - 265) * 2;
        let add: u16 = compressed_data.read(1)?;
        lenght = min_len + add;
    } else if curr <= 272 {
        let min_len: u16 = 19 + (curr - 269) * 4;
        let add: u16 = compressed_data.read(2)?;
        lenght = min_len + add;
    } else if curr <= 276 {
        let min_len: u16 = 35 + (curr - 273) * 8;
        let add: u16 = compressed_data.read(3)?;
        lenght = min_len + add;
    } else if curr <= 280 {
        let min_len: u16 = 67 + (curr - 277) * 16;
        let add: u16 = compressed_data.read(4)?;
        lenght = min_len + add;
    } else if curr <= 284 {
        let min_len: u16 = 131 + (curr - 281) * 32;
        let add: u16 = compressed_data.read(5)?;
        lenght = min_len + add;
    } else {
        lenght = 258;
    }
/*
      Extra           Extra               Extra
 Code Bits Dist  Code Bits   Dist     Code Bits Distance
 ---- ---- ----  ---- ----  ------    ---- ---- --------
   0   0    1     10   4     33-48    20    9   1025-1536
   1   0    2     11   4     49-64    21    9   1537-2048
   2   0    3     12   5     65-96    22   10   2049-3072
   3   0    4     13   5     97-128   23   10   3073-4096
   4   1   5,6    14   6    129-192   24   11   4097-6144
   5   1   7,8    15   6    193-256   25   11   6145-8192
   6   2   9-12   16   7    257-384   26   12  8193-12288
   7   2  13-16   17   7    385-512   27   12 12289-16384
   8   3  17-24   18   8    513-768   28   13 16385-24576
   9   3  25-32   19   8   769-1024   29   13 24577-32768
     */
    let tree: huffman::Node<u32> = huffman::generate_fixed_deflate_distance_tree(); //TODO: cache
    let (len2, currd): (u32, u32) = tree.read_one(compressed_data).unwrap();
    assert_eq!(len2, 5);
    let curr2 = currd as u16;

    if curr2 <= 3 {
        distance = curr2 + 1;
    } else {
        let extra_bits: u16 = (curr2 / 2) - 1;
        assert!(extra_bits <= 13);
        let base: u16 = curr2 + 1;
        let add: u16 = compressed_data.read(extra_bits.into())?;
        //dbg!(base, extra_bits, add);
        distance = base + add;
    }
    
    return Ok((lenght, distance));
}

fn find_lower_bound(raw: &Vec<LzssCode>, upper_pos: usize, b_distance: usize) -> usize {
    if upper_pos == 0 {
        return 0;
    }
    let mut count_r = 0;
    let mut low = 0;
    while count_r < b_distance {
        let v = &raw[upper_pos - low as usize];
        match v {
            LzssCode::Val {..} => {count_r += 1},
            LzssCode::Block {..} => {count_r += 2},
        }
        low += 1;
    }
    return upper_pos - low;
}

fn find_upper_bound(raw: &Vec<LzssCode>, lower_pos: usize, b_lenght: usize) -> usize {
    let mut count_r = 0;
    let mut up = 0;
    while count_r < b_lenght {
        let v = &raw[lower_pos + up as usize];
        match v {
            LzssCode::Val {..} => {count_r += 1},
            LzssCode::Block {lenght, ..} => {count_r += *lenght as usize},
        }
        up += 1;
    }
    return lower_pos + up;
}

fn decode_block(raw: &Vec<LzssCode>, pos: usize, b_lenght: usize, b_distance: usize) -> Vec<u8> {
    let mut ret: Vec<u8> = Vec::new();
    let size: usize = cmp::min(b_lenght, b_distance);

    let low = find_lower_bound(&raw, pos, b_distance) + 1;
    let up = find_upper_bound(&raw, low, size);

    let val = &raw[low .. up];
    dbg!(low..up);
    dbg!(val);
    let mut s = val.iter().cycle();
    let mut added_val = 0;
    let mut pos_block = 0;
    while added_val < b_lenght {
        match *s.next().unwrap() {
            LzssCode::Val {code} => {
                ret.push(code);
                added_val += 1;
            },
            LzssCode::Block {lenght, distance} => {
                let decoded_block = decode_block(&raw, pos_block, lenght as usize, distance as usize);
                for b in decoded_block {
                    if added_val < b_lenght {
                        ret.push(b);
                        added_val += 1;
                    }
                }
            }
        }
        pos_block = (pos_block + 1) % val.len();
    }
    dbg!(String::from_utf8_lossy(&ret));
    return ret;
}

pub fn lzss_decode(raw: &Vec<LzssCode>) -> Vec<u8> {
    dbg!(raw);
    let mut ret = Vec::with_capacity(raw.len());
    for (i, code) in raw.iter().enumerate() {
        match code {
            LzssCode::Val {code} => {
                ret.push(*code);
            },
            LzssCode::Block {lenght, distance} => {
                dbg!(lenght, distance);
                ret.extend(decode_block(&raw, i - 1, *lenght as usize, *distance as usize));
            }
                
        }
        dbg!(String::from_utf8_lossy(&ret));
    }
    return ret;
}
