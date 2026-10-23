use std::f32::consts::TAU;

use super::Pen;
use crate::geometry::{Point, Rect};

const FRAME_INSET: f32 = 0.07;
const FRAME_RADIUS: f32 = 0.18;
const SIDEBAR_SPLIT: f32 = 0.38;
const PANEL_ALPHA: f32 = 0.35;
const GRID_GAP: f32 = 0.14;
const GRID_RADIUS: f32 = 0.12;
const CHEVRON_LEFT: f32 = 0.2;
const CHEVRON_TIP: f32 = 0.46;
const CHEVRON_TOP: f32 = 0.28;
const CHEVRON_BOTTOM: f32 = 0.72;
const CURSOR_LINE_Y: f32 = 0.74;
const CURSOR_LINE_LEFT: f32 = 0.56;
const CURSOR_LINE_RIGHT: f32 = 0.84;
const GEAR_RING: f32 = 0.3;
const GEAR_HOLE: f32 = 0.12;
const GEAR_TOOTH_FROM: f32 = 0.24;
const GEAR_TOOTH_TO: f32 = 0.45;
const GEAR_TOOTH_WIDTH: f32 = 0.16;
const GEAR_TEETH: usize = 8;
const ARROW_TOP: f32 = 0.16;
const ARROW_TIP: f32 = 0.64;
const ARROW_WING: f32 = 0.22;
const ARROW_WING_Y: f32 = 0.42;
const TRAY_Y: f32 = 0.84;
const TRAY_REACH: f32 = 0.3;

pub(super) fn icon_frame(rect: Rect, size: f32) -> Rect {
    rect.inset((size * FRAME_INSET).round()).snapped()
}

pub(super) fn sidebar(pen: &mut Pen, rect: Rect, size: f32, open: bool) {
    let frame = icon_frame(rect, size);
    let split = pen.crisp(frame.x + frame.width * SIDEBAR_SPLIT);
    if open {
        let panel = Rect::from_edges(frame.x, frame.y, split, frame.bottom());
        let radius = size * FRAME_RADIUS;
        pen.scene.fill(panel, pen.color.faded(PANEL_ALPHA), radius);
    }
    pen.frame(frame, size * FRAME_RADIUS);
    pen.line(
        Point::new(split, frame.y),
        Point::new(split, frame.bottom()),
    );
}

pub(super) fn split(pen: &mut Pen, rect: Rect, size: f32, vertical: bool) {
    let frame = icon_frame(rect, size);
    pen.frame(frame, size * FRAME_RADIUS);
    let middle = frame.center();
    if vertical {
        let x = pen.crisp(middle.x);
        pen.line(Point::new(x, frame.y), Point::new(x, frame.bottom()));
    } else {
        let y = pen.crisp(middle.y);
        pen.line(Point::new(frame.x, y), Point::new(frame.right(), y));
    }
}

pub(super) fn terminal(pen: &mut Pen, rect: Rect, size: f32) {
    let at = |fx: f32, fy: f32| Point::new(rect.x + size * fx, rect.y + size * fy);
    let tip = at(CHEVRON_TIP, 0.5);
    pen.line(at(CHEVRON_LEFT, CHEVRON_TOP), tip);
    pen.line(tip, at(CHEVRON_LEFT, CHEVRON_BOTTOM));
    let base = pen.crisp(rect.y + size * CURSOR_LINE_Y);
    pen.line(
        Point::new(rect.x + size * CURSOR_LINE_LEFT, base),
        Point::new(rect.x + size * CURSOR_LINE_RIGHT, base),
    );
}

pub(super) fn grid(pen: &mut Pen, rect: Rect, size: f32) {
    let frame = icon_frame(rect, size);
    let gap = (size * GRID_GAP).round().max(1.0);
    let cell = ((frame.width - gap) / 2.0).floor();
    for row in 0..2 {
        for col in 0..2 {
            let x = frame.x + col as f32 * (cell + gap);
            let y = frame.y + row as f32 * (cell + gap);
            pen.frame(Rect::new(x, y, cell, cell), size * GRID_RADIUS);
        }
    }
}

pub(super) fn gear(pen: &mut Pen, rect: Rect, size: f32) {
    let center = rect.center();
    let ring = size * GEAR_RING;
    let body = Rect::new(center.x - ring, center.y - ring, ring * 2.0, ring * 2.0);
    pen.scene
        .border(body, pen.color, ring, ring - size * GEAR_HOLE);
    let width = (size * GEAR_TOOTH_WIDTH).max(pen.stroke);
    for tooth in 0..GEAR_TEETH {
        let angle = tooth as f32 * TAU / GEAR_TEETH as f32;
        let (sin, cos) = angle.sin_cos();
        let at = |reach: f32| Point::new(center.x + cos * reach, center.y + sin * reach);
        pen.scene.line(
            at(size * GEAR_TOOTH_FROM),
            at(size * GEAR_TOOTH_TO),
            width,
            pen.color,
        );
    }
}

pub(super) fn update(pen: &mut Pen, rect: Rect, size: f32) {
    let x = pen.crisp(rect.center().x);
    let y = |fraction: f32| rect.y + size * fraction;
    let tip = Point::new(x, y(ARROW_TIP));
    pen.line(Point::new(x, y(ARROW_TOP)), tip);
    for side in [-1.0, 1.0] {
        let wing = Point::new(x + side * size * ARROW_WING, y(ARROW_WING_Y));
        pen.line(wing, tip);
    }
    let tray = pen.crisp(y(TRAY_Y));
    let reach = size * TRAY_REACH;
    pen.line(Point::new(x - reach, tray), Point::new(x + reach, tray));
}
