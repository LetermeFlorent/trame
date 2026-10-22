use winit::window::Icon;

pub const SIZE: u32 = 64;
const BYTES_PER_PIXEL: usize = 4;
const PIXELS: &[u8] = include_bytes!("../assets/icon-64.rgba");

pub fn pixels() -> &'static [u8] {
    PIXELS
}

pub fn pixel(x: u32, y: u32) -> [u8; BYTES_PER_PIXEL] {
    let start = (y * SIZE + x) as usize * BYTES_PER_PIXEL;
    let mut rgba = [0; BYTES_PER_PIXEL];
    rgba.copy_from_slice(&PIXELS[start..start + BYTES_PER_PIXEL]);
    rgba
}

pub fn window_icon() -> Option<Icon> {
    Icon::from_rgba(PIXELS.to_vec(), SIZE, SIZE).ok()
}
