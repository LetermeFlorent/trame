use super::super::Ink;
use super::HEAVY_WIDTH;
use super::arm::Arm;
use crate::geometry::Rect;
use crate::render::Scene;

pub(super) const DOUBLE_WIDTH: f32 = 3.0;
pub(super) const DOUBLE_SECOND_STROKE: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Weight {
    None,
    Light,
    Heavy,
    Double,
}

impl Weight {
    fn from_nibble(code: u16, shift: u32) -> Self {
        match (code >> shift) & 0xF {
            1 => Self::Light,
            2 => Self::Heavy,
            3 => Self::Double,
            _ => Self::None,
        }
    }

    pub(super) fn thickness(self, light: f32) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Light => light,
            Self::Heavy => light * HEAVY_WIDTH,
            Self::Double => light * DOUBLE_WIDTH,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Span {
    pub(super) lo: f32,
    pub(super) size: f32,
}

impl Span {
    pub(super) fn centered(self, thickness: f32) -> f32 {
        self.lo + ((self.size - thickness) / 2.0).floor()
    }

    pub(super) fn center(self) -> f32 {
        (self.lo + self.size / 2.0).round()
    }

    pub(super) fn hi(self) -> f32 {
        self.lo + self.size
    }
}

pub fn draw_lines(scene: &mut Scene, code: u16, ink: Ink) {
    let Ink { rect, color, light } = ink;
    let [up, right, down, left] = [12, 8, 4, 0].map(|shift| Weight::from_nibble(code, shift));
    let xs = Span {
        lo: rect.x,
        size: rect.width,
    };
    let ys = Span {
        lo: rect.y,
        size: rect.height,
    };
    let arm = |weight, high, opposite, before, after| Arm {
        weight,
        high,
        opposite,
        before,
        after,
    };
    let arms = [
        (arm(left, false, right, up, down), false),
        (arm(right, true, left, up, down), false),
        (arm(up, false, down, left, right), true),
        (arm(down, true, up, left, right), true),
    ];
    for (arm, vertical) in arms {
        let (along, across) = if vertical { (ys, xs) } else { (xs, ys) };
        for [from, to, start, end] in arm.strokes(along, across, light) {
            let stroke = if vertical {
                Rect::from_edges(start, from, end, to)
            } else {
                Rect::from_edges(from, start, to, end)
            };
            scene.fill(stroke, color, 0.0);
        }
    }
}
