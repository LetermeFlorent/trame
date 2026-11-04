use std::hint::black_box;

pub fn wipe(bytes: &mut [u8]) {
    bytes.fill(0);
    black_box(bytes);
}
