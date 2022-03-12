use std::collections::vec_deque::VecDeque;

fn posl_substr(d: &[u8], w: &[u8]) -> Option<(usize, usize)> {
    let mut pos: usize = 0;
    let mut l: usize = 0;
    let mut found: bool = false;

    let wlenght = w.len();
    let dlenght = d.len();
    if wlenght == 0 || dlenght == 0 {
        return None;
    }

    let mut i = 0;
    while i < wlenght {
        if w[i] == d[0] {
            let mut j = 0;
            found = true;
            while i + j < wlenght && j < dlenght {
                if w[i + j] != d[j] {
                    break;
                }
                j += 1;
            }
            if j > l {
                l = j;
                pos = i;
            }
        }
        i += 1;
    }

    if ! found {
        return None;
    }

    return Some((pos, l));
}

fn compress(data: &[u8], window_size: usize) -> Vec<(usize, usize, u8)> {
    assert!(window_size >= 2);
    let mut d = &data[..];
    let mut window: VecDeque<u8> = VecDeque::with_capacity(window_size);
    let mut ret: Vec<(usize, usize, u8)> = Vec::new();

    while ! d.is_empty() {
        let start_of_match: usize;
        let lenght_of_match: usize;
        let c: u8;

        let sw = window.make_contiguous();
        if let Some(mat) = posl_substr(d, sw) {
            start_of_match = mat.0;
            if mat.1 < d.len() {
                lenght_of_match = mat.1;
                c = d[lenght_of_match];
            } else { // last byte included
                lenght_of_match = mat.1 - 1;
                c = d[lenght_of_match];
            }
        } else {
            start_of_match = 0;
            lenght_of_match = 0;
            c = d[0];
        }

        ret.push((start_of_match, lenght_of_match, c));

        //

        if lenght_of_match == d.len() {
            break;
        }

        let split_point = (lenght_of_match + 1) as usize;
        if window.len() + split_point > window_size {
            for _ in 0..split_point {
                window.pop_front();
            }
        }

        let n = &d[.. split_point];
        for c in n {
            window.push_back(*c);
        }

        d = &d[split_point ..];
    }
    return ret;
}

fn decompress(compressed_data: Vec<(usize, usize, u8)>, window_size: usize) -> Vec<u8> {
    assert!(window_size >= 2);
    let mut ret: Vec<u8> = Vec::with_capacity(compressed_data.len());
    let mut curr = 0;

    for (pos, len, d) in compressed_data {
        if len > 0 {
            for i in 0..len {
                ret.push(ret[curr + pos + i]);
            }
        }

        ret.push(d);

        if ret.len() > window_size {
            curr += len + 1;
        }
    }

    return ret;
}


#[test]
fn test_compress_decompress() {
    assert_eq!(decompress(compress(b"", 10), 10), b"");
    assert_eq!(decompress(compress(b"AABCBBABC", 10), 10), b"AABCBBABC");
    assert_eq!(decompress(compress(b"AABCBBABC", 3), 3), b"AABCBBABC");
    assert_eq!(decompress(compress(b"Hello friend, Hello world!", 5), 5), b"Hello friend, Hello world!");
    assert_eq!(decompress(compress(b"Hello friend, Hello world!", 100), 100), b"Hello friend, Hello world!");
}

#[test]
fn test_posl_substr() {
    assert_eq!(posl_substr(&[1, 2], &[3, 1, 3, 1, 2]),
               Some((3, 2)));

    assert_eq!(posl_substr(&[0, 1, 2], &[3, 1, 3, 1, 2]),
               None);

    assert_eq!(posl_substr(&[1, 2, 4], &[3, 1, 3, 1, 2]),
               Some((3, 2)));

    assert_eq!(posl_substr(&[3, 1], &[3, 1, 3, 1, 2]),
               Some((0, 2)));

    assert_eq!(posl_substr(&[1, 1], &[3, 1, 3, 1, 2]),
               Some((1, 1)));

    assert_eq!(posl_substr(&[], &[3, 1, 3, 1, 2]),
               None);

    assert_eq!(posl_substr(&[1, 2], &[]),
               None);
}
