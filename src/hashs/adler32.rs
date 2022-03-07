
const MOD_ADLER32: u32 = 65521;

struct Adler32 {
    s1: u32,
    s2: u32,
    count: u16,
}

impl Adler32 {
    fn new() -> Adler32 {
        Adler32 {
            s1: 1,
            s2: 0,
            count: 0
        }
    }

    fn update(&mut self, data: &[u8]) {
        for d in data {
            self.s1 = self.s1 + *d as u32;
            self.s2 = self.s2 + self.s1;

            self.count += 1;
            if self.count >= 5552 {
                self.s1 = self.s1 % MOD_ADLER32;
                self.s2 = self.s2 % MOD_ADLER32;
                self.count = 0;
            }
        }
    }

    fn finalise(&mut self) -> u32 {
        self.s1 = self.s1 % MOD_ADLER32;
        self.s2 = self.s2 % MOD_ADLER32;

        (self.s2 << 16) | self.s1 as u32
    }
}

#[test]
fn test_alder32_simple() {
    let mut h = Adler32::new();
    h.update(b"Hello");
    assert_eq!(h.finalise(), 0x058c01f5);
}

#[test]
fn test_alder32_double_finalise() {
    let mut h = Adler32::new();
    h.update(b"Hello");
    assert_eq!(h.finalise(), 0x058c01f5);
    assert_eq!(h.finalise(), 0x058c01f5);
}

#[test]
fn test_alder32_multi_update() {
    let mut h = Adler32::new();
    h.update(b"Hello");
    h.update(b" ");
    h.update(b"World!");
    assert_eq!(h.finalise(), 0x1c49043e);
}

#[test]
fn test_alder32_no_update() {
    let mut h = Adler32::new();
    assert_eq!(h.finalise(), 0x1);
}

#[test]
fn test_alder32_empty_update() {
    let mut h = Adler32::new();
    h.update(b"");
    assert_eq!(h.finalise(), 0x1);
}

#[test]
fn test_alder32_big_update() {
    let mut h = Adler32::new();
    h.update(&[b'a'; 255]);
    assert_eq!(h.finalise(), 0x534f60a0);
}

#[test]
fn test_alder32_big_update_rollover() {
    let mut h = Adler32::new();
    h.update(&[0xff; 255]);
    assert_eq!(h.finalise(), 0x08f0fe02);
}