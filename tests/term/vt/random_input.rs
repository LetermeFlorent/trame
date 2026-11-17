use std::io::Write;
use std::ops::RangeInclusive;

const SEED_SPREAD: u64 = 0x9E37_79B9_7F4A_7C15;
const XORSHIFT: (u32, u32, u32) = (13, 7, 17);
const MAX_FRAGMENTS: usize = 48;
const PRINTABLE: RangeInclusive<u8> = b' '..=b'~';
const NUMBERED_FINALS: &[u8] = b"GDCP@Xbd";
const PIECES: &[&[u8]] = &[
    b"\r",
    b"\n",
    b"\x08",
    b"\t",
    b"\x0e",
    b"\x0f",
    b"\x07",
    b"\x7f",
    b"\x18",
    b"\x1b(0",
    b"\x1b(B",
    b"\x1b)0",
    b"\x1b)A",
    b"\x1b[4h",
    b"\x1b[4l",
    b"\x1b[?7l",
    b"\x1b[?7h",
    b"\x1b[1;31m",
    b"\x1b[7;44m",
    b"\x1b[0m",
    b"\x1b]8;;https://a\x07",
    b"\x1b]8;;\x07",
    b"\x1b7",
    b"\x1b8",
    b"\x1b[K",
    b"\x1b[1K",
    b"\x1b[2;3r",
    b"\x1b[r",
    b"\x1b[?6h",
    b"\x1b[?6l",
    b"\x1b[?1049h",
    b"\x1b[?1049l",
    b"\x1b[!p",
    b"\xC3",
    b"\xE6\xBC",
    b"\xFF",
    "漢".as_bytes(),
    "漢\x1b[2D\x1b[X\x1b[D".as_bytes(),
    "한".as_bytes(),
    "😀".as_bytes(),
    "é".as_bytes(),
    "\u{301}".as_bytes(),
    "\u{308}".as_bytes(),
    "\u{200D}".as_bytes(),
];

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(SEED_SPREAD))
    }

    pub fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << XORSHIFT.0;
        self.0 ^= self.0 >> XORSHIFT.1;
        self.0 ^= self.0 << XORSHIFT.2;
        (self.0 % bound as u64) as usize
    }
}

pub fn random_input(rng: &mut Rng, rows: usize, cols: usize) -> Vec<u8> {
    let mut input = Vec::new();
    for _ in 0..=rng.below(MAX_FRAGMENTS) {
        match rng.below(5) {
            0 | 1 => {
                let len = rng.below(cols * 3 + 4) + 1;
                let first = *PRINTABLE.start();
                input.extend((0..len).map(|_| first + rng.below(PRINTABLE.len()) as u8));
            }
            2 | 3 => input.extend_from_slice(PIECES[rng.below(PIECES.len())]),
            _ => numbered(rng, &mut input, rows, cols),
        }
    }
    input
}

fn numbered(rng: &mut Rng, input: &mut Vec<u8>, rows: usize, cols: usize) {
    let count = rng.below(cols + 2) + 1;
    let row = rng.below(rows + 1) + 1;
    let pick = rng.below(NUMBERED_FINALS.len() + 2);
    match NUMBERED_FINALS.get(pick) {
        Some(&action) => write!(input, "\x1b[{count}{}", char::from(action)),
        None if pick == NUMBERED_FINALS.len() => write!(input, "\x1b[{row};{count}H"),
        None => write!(input, "\x1b[{row};{}r", rng.below(rows + 1) + 1),
    }
    .expect("a Vec grows without failing");
}
