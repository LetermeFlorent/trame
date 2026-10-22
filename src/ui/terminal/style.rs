use crate::color::Rgba;
use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::term::{Cell, Color, Flags};
use crate::text::CellMetrics;
use crate::theme::Theme;

const DIM_AMOUNT: f32 = 0.4;
const CURL_SEGMENTS: u8 = 4;
const MIN_CURL_AMPLITUDE: f32 = 1.5;

pub fn colors(theme: &Theme, cell: &Cell) -> (Rgba, Rgba) {
    let flags = cell.flags;
    let (mut foreground, background) = if flags.contains(Flags::INVERSE) {
        let foreground = match cell.bg {
            Color::Background => theme.terminal,
            other => theme.terminal_color(other),
        };
        let background = match cell.fg {
            Color::Foreground => theme.foreground,
            other => theme.terminal_color(other),
        };
        (foreground, background)
    } else {
        (theme.terminal_color(cell.fg), theme.terminal_color(cell.bg))
    };
    if flags.contains(Flags::DIM) {
        foreground = foreground.mix(background, DIM_AMOUNT);
    }
    if flags.contains(Flags::HIDDEN) {
        foreground = background;
    }
    (foreground, background)
}

pub fn draw_decorations(
    scene: &mut Scene,
    flags: Flags,
    cell: Rect,
    metrics: &CellMetrics,
    color: Rgba,
) {
    let thickness = metrics.underline_thickness.max(1.0);
    let underline_y = cell.y + metrics.underline_position;
    if flags.contains(Flags::UNDERLINE) {
        scene.fill(
            Rect::new(cell.x, underline_y, cell.width, thickness),
            color,
            0.0,
        );
    }
    if flags.contains(Flags::DOUBLE_UNDERLINE) {
        let second = (underline_y + thickness * 2.0).min(cell.bottom() - thickness);
        let first = (second - thickness * 2.0).min(underline_y);
        for y in [first, second] {
            scene.fill(Rect::new(cell.x, y, cell.width, thickness), color, 0.0);
        }
    }
    if flags.contains(Flags::CURLY_UNDERLINE) {
        draw_curl(scene, cell, underline_y, thickness, color);
    }
    if flags.contains(Flags::STRIKE) {
        let strike = metrics.strike_thickness.max(1.0);
        scene.fill(
            Rect::new(cell.x, cell.y + metrics.strike_position, cell.width, strike),
            color,
            0.0,
        );
    }
}

fn draw_curl(scene: &mut Scene, cell: Rect, y: f32, thickness: f32, color: Rgba) {
    let amplitude = thickness.max(MIN_CURL_AMPLITUDE);
    let middle = (y + thickness / 2.0).min(cell.bottom() - amplitude);
    let step = cell.width / f32::from(CURL_SEGMENTS);
    let height = |index: u8| match index % 4 {
        1 => -amplitude,
        3 => amplitude,
        _ => 0.0,
    };
    scene.push_clip(cell);
    for index in 0..CURL_SEGMENTS {
        let from = Point::new(cell.x + f32::from(index) * step, middle + height(index));
        let to = Point::new(from.x + step, middle + height(index + 1));
        scene.line(from, to, thickness, color);
    }
    scene.pop_clip();
}
