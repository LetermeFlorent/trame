use crate::theme::Rgba;

const CUBE_START: u8 = 16;
const GRAY_START: u8 = 232;
const CUBE_SIDE: u8 = 6;
const CUBE_PLANE: u8 = CUBE_SIDE * CUBE_SIDE;
const GRAY_BASE: u8 = 8;
const GRAY_STEP: u8 = 10;
const CUBE_LEVELS: [u8; CUBE_SIDE as usize] = [0, 95, 135, 175, 215, 255];

pub fn extended(index: u8) -> Rgba {
    if index >= GRAY_START {
        let level = GRAY_BASE + (index - GRAY_START) * GRAY_STEP;
        return Rgba::rgb8(level, level, level);
    }
    let cube = index.saturating_sub(CUBE_START);
    let level = |value: u8| CUBE_LEVELS[usize::from(value % CUBE_SIDE)];
    Rgba::rgb8(
        level(cube / CUBE_PLANE),
        level(cube / CUBE_SIDE),
        level(cube),
    )
}
