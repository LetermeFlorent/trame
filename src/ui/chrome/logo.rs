use winit::window::Icon;

use crate::geometry::Rect;
use crate::render::Scene;
use crate::theme::Rgba;

pub const SIZE: u32 = 64;
const PIXELS: &[u8] = include_bytes!("../../../assets/icon-64.rgba");

const VIEW: f32 = 240.0;
const THREAD: f32 = 28.0;
const WARP: Rgba = Rgba::hex(0x3b7ddd);
const WEFT: Rgba = Rgba::hex(0xe8a33d);

struct Strand {
    vertical: bool,
    across: f32,
    from: f32,
    to: f32,
    round_start: bool,
}

const fn strand(vertical: bool, across: f32, from: f32, to: f32, round_start: bool) -> Strand {
    Strand {
        vertical,
        across,
        from,
        to,
        round_start,
    }
}

const STRANDS: [Strand; 8] = [
    strand(true, 66.0, 22.0, 137.0, true),
    strand(true, 66.0, 183.0, 218.0, false),
    strand(true, 146.0, 22.0, 57.0, true),
    strand(true, 146.0, 103.0, 218.0, false),
    strand(false, 66.0, 22.0, 57.0, true),
    strand(false, 66.0, 103.0, 218.0, false),
    strand(false, 146.0, 22.0, 137.0, true),
    strand(false, 146.0, 183.0, 218.0, false),
];

pub fn pixels() -> &'static [u8] {
    PIXELS
}

pub fn window_icon() -> Option<Icon> {
    Icon::from_rgba(PIXELS.to_vec(), SIZE, SIZE).ok()
}

pub fn draw_logo(scene: &mut Scene, rect: Rect) {
    let unit = rect.width.min(rect.height) / VIEW;
    let place = |along: f32, across: f32, length: f32, vertical: bool| {
        let (x, y, width, height) = match vertical {
            true => (across, along, THREAD, length),
            false => (along, across, length, THREAD),
        };
        Rect::new(
            rect.x + x * unit,
            rect.y + y * unit,
            width * unit,
            height * unit,
        )
    };
    let cap = THREAD / 2.0;
    for strand in &STRANDS {
        let color = if strand.vertical { WARP } else { WEFT };
        let whole = place(
            strand.from,
            strand.across,
            strand.to - strand.from,
            strand.vertical,
        );
        scene.fill(whole, color, cap * unit);
        let square = if strand.round_start {
            strand.to - cap
        } else {
            strand.from
        };
        scene.fill(
            place(square, strand.across, cap, strand.vertical),
            color,
            0.0,
        );
    }
}
