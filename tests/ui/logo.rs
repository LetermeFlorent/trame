use trame::ui::chrome::logo::{SIZE, pixels, window_icon};

const BYTES_PER_PIXEL: usize = 4;
const CENTER: u32 = SIZE / 2;

fn pixel(x: u32, y: u32) -> &'static [u8] {
    let start = (y * SIZE + x) as usize * BYTES_PER_PIXEL;
    &pixels()[start..start + BYTES_PER_PIXEL]
}

#[test]
fn embedded_pixels_cover_the_whole_square() {
    assert_eq!(pixels().len(), (SIZE * SIZE) as usize * BYTES_PER_PIXEL);
}

#[test]
fn corners_are_fully_transparent() {
    let last = SIZE - 1;
    for (x, y) in [(0, 0), (last, 0), (0, last), (last, last)] {
        assert_eq!(pixel(x, y)[3], 0, "corner ({x}, {y})");
    }
}

#[test]
fn a_thread_pixel_is_opaque_blue() {
    let thread_x = SIZE / 3;
    assert_eq!(pixel(thread_x, CENTER), [0x3b, 0x7d, 0xdd, 0xff]);
}

#[test]
fn winit_accepts_the_pixels() {
    assert!(window_icon().is_some());
}
