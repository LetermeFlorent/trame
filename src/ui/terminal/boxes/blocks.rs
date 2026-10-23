use crate::geometry::Rect;
use crate::render::Scene;
use crate::theme::Rgba;

const UPPER_LEFT: u8 = 1;
const UPPER_RIGHT: u8 = 2;
const LOWER_LEFT: u8 = 4;
const LOWER_RIGHT: u8 = 8;
const SHADE_STEP: f32 = 0.25;

pub fn draw_block(scene: &mut Scene, ch: char, rect: Rect, color: Rgba) -> bool {
    let code = u32::from(ch);
    let eighths = |total: f32, count: u32| (total * count as f32 / 8.0).round();
    let part = match code {
        0x2580 => Rect::new(rect.x, rect.y, rect.width, eighths(rect.height, 4)),
        0x2581..=0x2588 => {
            let height = eighths(rect.height, code - 0x2580);
            Rect::new(rect.x, rect.bottom() - height, rect.width, height)
        }
        0x2589..=0x258F => Rect::new(
            rect.x,
            rect.y,
            eighths(rect.width, 8 - (code - 0x2588)),
            rect.height,
        ),
        0x2590 => {
            let width = eighths(rect.width, 4);
            Rect::new(rect.right() - width, rect.y, width, rect.height)
        }
        0x2591..=0x2593 => {
            let alpha = (code - 0x2590) as f32 * SHADE_STEP;
            scene.fill(rect, color.faded(alpha), 0.0);
            return true;
        }
        0x2594 => Rect::new(rect.x, rect.y, rect.width, eighths(rect.height, 1)),
        0x2595 => {
            let width = eighths(rect.width, 1);
            Rect::new(rect.right() - width, rect.y, width, rect.height)
        }
        0x2596..=0x259F => {
            draw_quadrants(scene, quadrants(code), rect, color);
            return true;
        }
        _ => return false,
    };
    scene.fill(part, color, 0.0);
    true
}

fn quadrants(code: u32) -> u8 {
    match code {
        0x2596 => LOWER_LEFT,
        0x2597 => LOWER_RIGHT,
        0x2598 => UPPER_LEFT,
        0x2599 => UPPER_LEFT | LOWER_LEFT | LOWER_RIGHT,
        0x259A => UPPER_LEFT | LOWER_RIGHT,
        0x259B => UPPER_LEFT | UPPER_RIGHT | LOWER_LEFT,
        0x259C => UPPER_LEFT | UPPER_RIGHT | LOWER_RIGHT,
        0x259D => UPPER_RIGHT,
        0x259E => UPPER_RIGHT | LOWER_LEFT,
        _ => UPPER_RIGHT | LOWER_LEFT | LOWER_RIGHT,
    }
}

fn draw_quadrants(scene: &mut Scene, mask: u8, rect: Rect, color: Rgba) {
    let middle_x = rect.x + (rect.width / 2.0).round();
    let middle_y = rect.y + (rect.height / 2.0).round();
    let parts = [
        (
            UPPER_LEFT,
            Rect::from_edges(rect.x, rect.y, middle_x, middle_y),
        ),
        (
            UPPER_RIGHT,
            Rect::from_edges(middle_x, rect.y, rect.right(), middle_y),
        ),
        (
            LOWER_LEFT,
            Rect::from_edges(rect.x, middle_y, middle_x, rect.bottom()),
        ),
        (
            LOWER_RIGHT,
            Rect::from_edges(middle_x, middle_y, rect.right(), rect.bottom()),
        ),
    ];
    for (bit, part) in parts {
        if mask & bit != 0 {
            scene.fill(part, color, 0.0);
        }
    }
}
