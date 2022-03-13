use std::collections::vec_deque::VecDeque;

fn posl_substr_overlap(d: &[u8], w: &[u8]) -> Option<(usize, usize)> {
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
            while j < dlenght {
                if w[i + (j % (wlenght - i))] != d[j] {
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
        if let Some(mat) = posl_substr_overlap(d, sw) {
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
    assert_eq!(decompress(compress(b"Blah blah blah blah blah!", 100), 100), b"Blah blah blah blah blah!");
    assert_eq!(decompress(compress(b"Blah blah blah blah blah!", 5), 5), b"Blah blah blah blah blah!");
}

#[test]
fn test_compress_simple() {
    assert_eq!(compress(b"AABCBBABC", 50),
               vec!((0, 0, 65), (0, 1, 66), (0, 0, 67), (2, 1, 66), (1, 2, 67)));
    assert_eq!(compress(b"Hello friend, Hello world!", 50),
               vec!((0, 0, 72), (0, 0, 101), (0, 0, 108), (2, 1, 111), (0, 0, 32), (0, 0, 102), (0, 0, 114), (0, 0, 105), (1, 1, 110), (0, 0, 100), (0, 0, 44), (5, 1, 72), (1, 5, 119), (4, 1, 114), (2, 1, 100), (0, 0, 33)));
}

#[test]
fn test_compress_overlapping() {
    let c = compress(b"Blah blah blah blah blah!", 6);
    assert_eq!(c, vec!((0, 0, 66), (0, 0, 108), (0, 0, 97), (0, 0, 104), (0, 0, 32), (0, 0, 98), (1, 18, 33)));
    let c = compress(b"Blah blah blah blah blah!", 5);
    assert_eq!(c, vec!((0, 0, 66), (0, 0, 108), (0, 0, 97), (0, 0, 104), (0, 0, 32), (0, 0, 98), (0, 18, 33)));

}

#[test]
fn test_posl_substr_overlap() {
    assert_eq!(posl_substr_overlap(&[1, 2], &[3, 1, 3, 1, 2]),
               Some((3, 2)));

    assert_eq!(posl_substr_overlap(&[0, 1, 2], &[3, 1, 3, 1, 2]),
               None);

    assert_eq!(posl_substr_overlap(&[1, 2, 4], &[3, 1, 3, 1, 2]),
               Some((3, 2)));

    assert_eq!(posl_substr_overlap(&[3, 1], &[3, 1, 3, 1, 2]),
               Some((0, 2)));

    assert_eq!(posl_substr_overlap(&[1, 1], &[3, 1, 3, 1, 2]),
               Some((1, 1)));

    assert_eq!(posl_substr_overlap(&[], &[3, 1, 3, 1, 2]),
               None);

    assert_eq!(posl_substr_overlap(&[1, 2], &[]),
               None);

    assert_eq!(posl_substr_overlap(&[1, 2, 1, 2], &[0, 1, 2]),
               Some((1, 4)));
}
