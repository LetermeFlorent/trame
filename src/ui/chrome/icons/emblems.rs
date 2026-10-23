use std::f32::consts::TAU;

use super::Pen;
use super::panels::{grid, icon_frame, terminal};
use crate::geometry::{Point, Rect};
use crate::theme::Emblem;

const FOLDER_TOP: f32 = 0.34;
const FOLDER_TAB_TOP: f32 = 0.18;
const FOLDER_TAB_END: f32 = 0.42;
const FOLDER_TAB_SLOPE: f32 = 0.52;
const CORNER: f32 = 0.12;
const CODE_REACH: f32 = 0.2;
const CODE_SIDE: f32 = 0.12;
const CODE_HALF_HEIGHT: f32 = 0.2;
const SLASH_TOP: f32 = 0.58;
const SLASH_BOTTOM: f32 = 0.42;
const SLASH_REACH: f32 = 0.22;
const SERVER_SPLIT: f32 = 0.06;
const LIGHT_LEFT: f32 = 0.24;
const LIGHT_FACTOR: f32 = 1.6;
const MERIDIAN_WIDTH: f32 = 0.42;
const DISC_INSET: f32 = 0.16;
const CAP_HEIGHT: f32 = 0.28;
const BELT_Y: f32 = 0.64;
const GEAR_RING: f32 = 0.27;
const GEAR_REACH: f32 = 0.46;
const GEAR_TEETH: usize = 8;
const TOOTH_FACTOR: f32 = 1.7;

pub(super) fn emblem(pen: &mut Pen, emblem: Emblem, rect: Rect, size: f32) {
    match emblem {
        Emblem::Grid => grid(pen, rect, size),
        Emblem::Terminal => terminal(pen, rect, size),
        Emblem::Folder => folder(pen, rect, size),
        Emblem::Code => code(pen, rect, size),
        Emblem::Server => server(pen, rect, size),
        Emblem::Globe => globe(pen, rect, size),
        Emblem::Database => database(pen, rect, size),
        Emblem::Gear => gear(pen, rect, size),
        Emblem::Letters(_) | Emblem::Image(_) => grid(pen, rect, size),
    }
}

fn folder(pen: &mut Pen, rect: Rect, size: f32) {
    let frame = icon_frame(rect, size);
    let top = pen.crisp(rect.y + size * FOLDER_TOP);
    let body = Rect::from_edges(frame.x, top, frame.right(), frame.bottom());
    pen.frame(body, size * CORNER);
    let left = pen.crisp(frame.x);
    let tab = pen.crisp(rect.y + size * FOLDER_TAB_TOP);
    let corner = Point::new(rect.x + size * FOLDER_TAB_END, tab);
    pen.line(Point::new(left, top), Point::new(left, tab));
    pen.line(Point::new(left, tab), corner);
    pen.line(corner, Point::new(rect.x + size * FOLDER_TAB_SLOPE, top));
}

fn code(pen: &mut Pen, rect: Rect, size: f32) {
    let center = rect.center();
    let half = size * CODE_HALF_HEIGHT;
    for side in [-1.0, 1.0] {
        let tip = Point::new(center.x + side * size * (0.5 - CODE_SIDE), center.y);
        let base = center.x + side * size * (0.5 - CODE_SIDE - CODE_REACH);
        pen.line(Point::new(base, center.y - half), tip);
        pen.line(tip, Point::new(base, center.y + half));
    }
    let reach = size * SLASH_REACH;
    pen.line(
        Point::new(rect.x + size * SLASH_TOP, center.y - reach),
        Point::new(rect.x + size * SLASH_BOTTOM, center.y + reach),
    );
}

fn server(pen: &mut Pen, rect: Rect, size: f32) {
    let frame = icon_frame(rect, size);
    let gap = (size * SERVER_SPLIT).round().max(1.0);
    let height = ((frame.height - gap) / 2.0).floor();
    let light = (pen.stroke * LIGHT_FACTOR).round().max(2.0);
    for row in 0..2 {
        let y = frame.y + row as f32 * (height + gap);
        let unit = Rect::new(frame.x, y, frame.width, height);
        pen.frame(unit, size * CORNER);
        let dot = Rect::new(
            (rect.x + size * LIGHT_LEFT).round(),
            (unit.center().y - light / 2.0).round(),
            light,
            light,
        );
        pen.scene.fill(dot, pen.color, light / 2.0);
    }
}

fn globe(pen: &mut Pen, rect: Rect, size: f32) {
    let disc = icon_frame(rect, size);
    pen.frame(disc, size);
    let width = (disc.width * MERIDIAN_WIDTH).round();
    let meridian = Rect::new(disc.center().x - width / 2.0, disc.y, width, disc.height);
    pen.frame(meridian, size);
    let y = pen.crisp(disc.center().y);
    pen.line(Point::new(disc.x, y), Point::new(disc.right(), y));
}

fn database(pen: &mut Pen, rect: Rect, size: f32) {
    let frame = icon_frame(rect, size);
    let inset = (size * DISC_INSET).round();
    let body = Rect::from_edges(
        frame.x + inset,
        frame.y,
        frame.right() - inset,
        frame.bottom(),
    );
    pen.frame(body, body.width * CORNER * 2.0);
    let cap = Rect::new(body.x, body.y, body.width, (size * CAP_HEIGHT).round());
    pen.frame(cap, size);
    let y = pen.crisp(rect.y + size * BELT_Y);
    pen.line(Point::new(body.x, y), Point::new(body.right(), y));
}

fn gear(pen: &mut Pen, rect: Rect, size: f32) {
    let center = Point::new(pen.crisp(rect.center().x), pen.crisp(rect.center().y));
    let ring = size * GEAR_RING;
    let hub = Rect::new(center.x - ring, center.y - ring, ring * 2.0, ring * 2.0);
    pen.frame(hub, size);
    let reach = size * GEAR_REACH;
    let width = pen.stroke * TOOTH_FACTOR;
    for tooth in 0..GEAR_TEETH {
        let angle = tooth as f32 * TAU / GEAR_TEETH as f32;
        let (sin, cos) = angle.sin_cos();
        let from = Point::new(center.x + cos * ring, center.y + sin * ring);
        let to = Point::new(center.x + cos * reach, center.y + sin * reach);
        pen.scene.line(from, to, width, pen.color);
    }
}
