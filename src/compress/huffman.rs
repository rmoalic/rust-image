
const MAX_BITS: usize = 10;

const DEFLATE_HUFFMAN_FIXED_CODE_VALUE: [u32;288] = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 421, 422, 423, 424, 425, 426, 427, 428, 429, 430, 431, 432, 433, 434, 435, 436, 437, 438, 439, 440, 441, 442, 443, 444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 455, 456, 457, 458, 459, 460, 461, 462, 463, 464, 465, 466, 467, 468, 469, 470, 471, 472, 473, 474, 475, 476, 477, 478, 479, 480, 481, 482, 483, 484, 485, 486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 500, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 192, 193, 194, 195, 196, 197, 198, 199];

fn gen_bl_count(code_lenghts: &Vec<u8>) -> Vec<u32> {
   let mut bl_count: Vec<u32> = vec!(0; MAX_BITS);

    for cl in code_lenghts {
        bl_count[*cl as usize] += 1;
    }
    return bl_count;
}

fn gen_next_code(bl_count2: &Vec<u32>) -> Vec<u32> {
    let mut next_code = vec!(0; MAX_BITS);// Vec::with_capacity(9);
    let mut code = 0;
    let mut bl_count = bl_count2.clone();
    
    bl_count[0 as usize] = 0;
    for bits in 1..MAX_BITS {
        code = (code + bl_count[bits-1]) << 1;
        if bl_count[bits] != 0 {
            next_code[bits as usize] = code;
        }
    }
    
    return next_code;
}

fn gen_code_values(bit_lenght: &Vec<u8>, next_code: &Vec<u32>, alphabet_size: usize) -> Vec<u32> {
    let mut tree: Vec<u32> = vec!(0; alphabet_size);
    let mut nc = next_code.clone();

    for n in 0..alphabet_size {
        let len = bit_lenght[n];
        if len != 0 {
            tree[n] = nc[len as usize];
            nc[len as usize] += 1;
        }
    }
    return tree;
}

fn generate_tree(code_lenghts: Vec<u8>) {
    let bl_count = gen_bl_count(&code_lenghts);
    let next_code = gen_next_code(&bl_count);
    let _code_values = gen_code_values(&code_lenghts, &next_code, code_lenghts.len());

}

#[test]
fn test_gen_bl_count() {
    let bl = vec!(3, 3, 3, 3, 3, 2, 4, 4);
    let bl_count = gen_bl_count(&bl);

    assert_eq!(bl_count, vec!(0, 0, 1, 5, 2, 0, 0, 0, 0, 0));
}

#[test]
fn test_gen_next_code() {
    let bl_count = vec!(0, 0, 1, 5, 2, 0, 0, 0, 0, 0);
    let next_code = gen_next_code(&bl_count);
    assert_eq!(next_code, vec!(0, 0, 0, 2, 14, 0, 0, 0, 0, 0));
}

#[test]
fn test_gen_code_values() {
    let bl = vec!(3, 3, 3, 3, 3, 2, 4, 4);
    let next_code = vec!(0, 0, 0, 2, 14, 0, 0, 0, 0);
    let code_values = gen_code_values(&bl, &next_code, 8);

    assert_eq!(code_values, vec!(2, 3, 4, 5, 6, 0, 14, 15));
}

#[test]
fn test_gen_deflate_fixed_code_values() {
    let mut bl: Vec<u8> = Vec::with_capacity(288);
    bl.extend(vec!(8; 144));
    bl.extend(vec!(9; 112));
    bl.extend(vec!(7; 24));
    bl.extend(vec!(8; 8));
    
    let bl_count = gen_bl_count(&bl);
    let next_code = gen_next_code(&bl_count);
    let code_values = gen_code_values(&bl, &next_code, 288);
    println!("{:?}", &code_values);
    assert_eq!(code_values[0]  , 0b000110000);
    assert_eq!(code_values[143], 0b010111111);
    assert_eq!(code_values[144], 0b110010000);
    assert_eq!(code_values[255], 0b111111111);
    assert_eq!(code_values[256], 0b000000000);
    assert_eq!(code_values[279], 0b000010111);
    assert_eq!(code_values[280], 0b011000000);
    assert_eq!(code_values[287], 0b011000111);
    assert_eq!(code_values, DEFLATE_HUFFMAN_FIXED_CODE_VALUE);
}

#[derive(Debug)]
enum Node<T> {
    Branch {
        left : Box<Node<T>>,
        right: Box<Node<T>>
    },
    Leaf {
        val: T
    },
    None
}

impl<T: Copy + PartialEq + std::fmt::Debug> Node<T> {

    fn new() -> Self {
        Node::None
    }

    fn insert(&mut self, branch: u32, nval: T) {
        let lead = branch.leading_zeros();
        /*if lead == 32 {
            assert!(3 == 4);
        }*/

        println!("\n{:b}", branch);
        let mut curr = self;
        let mut i = 32 - lead + 1;
        while i > 0 {            
            match curr {
                Node::Leaf {val} => {
                    assert_eq!(*val,  nval);
                    i -= 1;
                },
                Node::None => {
                    let new;
                    if i == 1 {
                        println!("> Added Value {:?}", nval);
                        new = Box::new(Node::Leaf {val: nval});
                    } else {
                        println!("> Added Node");
                        new = Box::new(Node::Branch { left: Box::new(Node::None), right: Box::new(Node::None) });
                    }
                    *curr = *new;
                }
                Node::Branch { ref mut left, ref mut right } => {
                    let a: bool = (branch & (1 << i - 2)) == 0;

                    println!("| move {}", if a {"r"} else {"l"});
                    curr = if a { right } else { left };
                    i -= 1;
                },
            }
        }
    }
}

#[test]
fn tree() {
    let mut t: Node<u32> = Node::new();

    for (i, v) in DEFLATE_HUFFMAN_FIXED_CODE_VALUE.iter().enumerate() {
        t.insert(*v, i as u32);
    }
    
    println!("{:?}", t);
    assert!(false);
}

/*
#[derive(Debug)]
struct Node<T> {
    val: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T: PartialOrd> Node<T> {

    fn new(val: T) -> Self {
        Node {
            left: None,
            right: None,
            val: val,
        }
    }

    fn insert(&mut self, val: T) {
        if self.val == val {
            return;
        }

        let node = if self.val < val { &mut self.left } else { &mut self.right };

        match node {
            Some(sub) => sub.insert(val),
            None => {
                let n = Node::new(val);
                *node = Some(Box::new(n));
            }
        }
    }
}

#[test]
fn test_node_insert() {
    let mut n: Node<u8> = Node::new(5);
    n.insert(1);
    n.insert(2);
    n.insert(4);
    n.insert(5);
    n.insert(6);
    println!("{:?}", n);
/*    assert_eq!(n,
               Node {
                   val: 5,
                   left: Node {
                       val: 
                   }
               }
               );*/
    assert!(false);
}
*/
