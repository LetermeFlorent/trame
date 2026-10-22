use super::super::Ink;
use super::HEAVY_WIDTH;
use crate::geometry::{Point, Rect};
use crate::render::Scene;

const DASH_RATIO: f32 = 0.6;

pub fn draw_dashes(scene: &mut Scene, code: u16, count: u8, ink: Ink) {
    let Ink { rect, color, light } = ink;
    let horizontal = code & 0x0F00 != 0;
    let heavy = code & 0x2222 != 0;
    let thickness = if heavy { light * HEAVY_WIDTH } else { light };
    let length = if horizontal { rect.width } else { rect.height };
    let slot = length / f32::from(count);
    let dash = (slot * DASH_RATIO).round().max(1.0);
    for index in 0..count {
        let offset = (f32::from(index) * slot + (slot - dash) / 2.0).round();
        let stroke = if horizontal {
            let y = rect.y + ((rect.height - thickness) / 2.0).floor();
            Rect::new(rect.x + offset, y, dash, thickness)
        } else {
            let x = rect.x + ((rect.width - thickness) / 2.0).floor();
            Rect::new(x, rect.y + offset, thickness, dash)
        };
        scene.fill(stroke, color, 0.0);
    }
}

pub fn draw_arc(scene: &mut Scene, code: u16, ink: Ink) {
    let Ink { rect, color, light } = ink;
    let up = code & 0xF000 != 0;
    let right = code & 0x0F00 != 0;
    let x = rect.x + ((rect.width - light) / 2.0).floor();
    let y = rect.y + ((rect.height - light) / 2.0).floor();
    let reach_x = rect.width * 2.0;
    let reach_y = rect.height * 2.0;
    let (left, right_edge) = if right {
        (x, x + reach_x)
    } else {
        (x + light - reach_x, x + light)
    };
    let (top, bottom) = if up {
        (y + light - reach_y, y + light)
    } else {
        (y, y + reach_y)
    };
    let radius = (rect.width.min(rect.height) / 2.0).round();
    scene.push_clip(rect);
    scene.border(
        Rect::from_edges(left, top, right_edge, bottom),
        color,
        radius,
        light,
    );
    scene.pop_clip();
}

pub fn draw_diagonals(scene: &mut Scene, ink: Ink, rising: bool, falling: bool) {
    let Ink { rect, color, light } = ink;
    scene.push_clip(rect);
    if rising {
        scene.line(
            Point::new(rect.x, rect.bottom()),
            Point::new(rect.right(), rect.y),
            light,
            color,
        );
    }
    if falling {
        scene.line(
            Point::new(rect.x, rect.y),
            Point::new(rect.right(), rect.bottom()),
            light,
            color,
        );
    }
    scene.pop_clip();
}
