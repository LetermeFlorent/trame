use crate::color::Rgba;

const CUBE_START: u8 = 16;
const GRAY_START: u8 = 232;
const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

pub fn extended(index: u8) -> Rgba {
    if index >= GRAY_START {
        let level = 8 + (index - GRAY_START) * 10;
        return Rgba::rgb8(level, level, level);
    }
    let cube = index.saturating_sub(CUBE_START);
    let level = |value: u8| CUBE_LEVELS[usize::from(value % 6)];
    Rgba::rgb8(level(cube / 36), level(cube / 6), level(cube))
}
