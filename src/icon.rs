use winit::window::Icon;

pub const SIZE: u32 = 64;
const PIXELS: &[u8] = include_bytes!("../assets/icon-64.rgba");

pub fn pixels() -> &'static [u8] {
    PIXELS
}

pub fn window_icon() -> Option<Icon> {
    Icon::from_rgba(PIXELS.to_vec(), SIZE, SIZE).ok()
}
