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
const START: f32 = 22.0;
const END: f32 = 218.0;
const GHOST: f32 = 0.28;
const STAGGER: f32 = 0.07;
const SWEEP: f32 = 0.26;
const RELEASE: f32 = 0.5;

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
    for strand in &STRANDS {
        let color = if strand.vertical { WARP } else { WEFT };
        draw_strand(scene, rect, strand, (strand.from, strand.to), color);
    }
}

pub fn draw_weaving(scene: &mut Scene, rect: Rect, color: Rgba, phase: f32) {
    for strand in &STRANDS {
        let whole = (strand.from, strand.to);
        draw_strand(scene, rect, strand, whole, color.faded(GHOST));
        let delay = turn(strand) * STAGGER;
        let from = strand.from.max(reach(phase - RELEASE - delay));
        let to = strand.to.min(reach(phase - delay));
        if to > from {
            draw_strand(scene, rect, strand, (from, to), color);
        }
    }
}

fn turn(strand: &Strand) -> f32 {
    let second = if strand.across > VIEW / 2.0 { 2.0 } else { 0.0 };
    second + if strand.vertical { 0.0 } else { 1.0 }
}

fn reach(time: f32) -> f32 {
    let t = (time / SWEEP).clamp(0.0, 1.0);
    START + (END - START) * t * t * (3.0 - 2.0 * t)
}

fn draw_strand(scene: &mut Scene, rect: Rect, strand: &Strand, span: (f32, f32), color: Rgba) {
    let unit = rect.width.min(rect.height) / VIEW;
    let place = |along: f32, length: f32| {
        let (x, y, width, height) = match strand.vertical {
            true => (strand.across, along, THREAD, length),
            false => (along, strand.across, length, THREAD),
        };
        Rect::new(
            rect.x + x * unit,
            rect.y + y * unit,
            width * unit,
            height * unit,
        )
    };
    let (from, to) = span;
    let cap = THREAD / 2.0;
    let (start, end) = match strand.round_start {
        true if to >= strand.to => (from, to + cap),
        false if from <= strand.from => (from - cap, to),
        _ => (from, to),
    };
    scene.push_clip(place(from, to - from));
    scene.fill(place(start, end - start), color, cap * unit);
    scene.pop_clip();
}
