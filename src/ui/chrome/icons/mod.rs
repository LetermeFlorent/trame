mod emblems;
mod panels;
mod tools;

use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::theme::{Emblem, Rgba};
use emblems::emblem;
use panels::{gear, sidebar, split, terminal, update};
use tools::pencil;

const PLUS_REACH: f32 = 0.36;
const CROSS_REACH: f32 = 0.28;
const DASH_REACH: f32 = 0.36;
const RESTORE_OFFSET: f32 = 0.2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Sidebar { open: bool },
    Plus,
    Close,
    SplitRight,
    SplitDown,
    Minimize,
    Maximize,
    Restore,
    Terminal,
    Gear,
    Update,
    Pencil,
    Emblem(Emblem),
}

pub struct Pen<'a> {
    pub scene: &'a mut Scene,
    pub color: Rgba,
    pub stroke: f32,
}

impl Pen<'_> {
    fn line(&mut self, from: Point, to: Point) {
        self.scene.line(from, to, self.stroke, self.color);
    }

    fn frame(&mut self, rect: Rect, radius: f32) {
        self.scene.border(rect, self.color, radius, self.stroke);
    }

    fn crisp(&self, value: f32) -> f32 {
        (value - self.stroke / 2.0).round() + self.stroke / 2.0
    }
}

pub fn draw_icon(pen: &mut Pen, icon: Icon, rect: Rect) {
    let size = rect.width.min(rect.height);
    let center = Point::new(pen.crisp(rect.center().x), pen.crisp(rect.center().y));
    match icon {
        Icon::Sidebar { open } => sidebar(pen, rect, size, open),
        Icon::Plus => cross(pen, center, size * PLUS_REACH, false),
        Icon::Close => cross(pen, center, size * CROSS_REACH, true),
        Icon::SplitRight => split(pen, rect, size, true),
        Icon::SplitDown => split(pen, rect, size, false),
        Icon::Minimize => {
            let reach = size * DASH_REACH;
            pen.line(
                Point::new(center.x - reach, center.y),
                Point::new(center.x + reach, center.y),
            );
        }
        Icon::Maximize => pen.frame(rect.snapped(), 0.0),
        Icon::Restore => restore(pen, rect.snapped(), size),
        Icon::Terminal => terminal(pen, rect, size),
        Icon::Gear => gear(pen, rect, size),
        Icon::Update => update(pen, rect, size),
        Icon::Pencil => pencil(pen, center, size),
        Icon::Emblem(kind) => emblem(pen, kind, rect, size),
    }
}

fn cross(pen: &mut Pen, center: Point, reach: f32, diagonal: bool) {
    let point = |dx: f32, dy: f32| Point::new(center.x + dx, center.y + dy);
    if diagonal {
        pen.line(point(-reach, -reach), point(reach, reach));
        pen.line(point(-reach, reach), point(reach, -reach));
    } else {
        pen.line(point(-reach, 0.0), point(reach, 0.0));
        pen.line(point(0.0, -reach), point(0.0, reach));
    }
}

fn restore(pen: &mut Pen, rect: Rect, size: f32) {
    let offset = (size * RESTORE_OFFSET).round().max(2.0);
    let front = Rect::from_edges(
        rect.x,
        rect.y + offset,
        rect.right() - offset,
        rect.bottom(),
    );
    pen.frame(front, 0.0);
    let half = pen.stroke / 2.0;
    let left = rect.x + offset + half;
    let top = rect.y + half;
    let right = rect.right() - half;
    let bottom = rect.bottom() - offset - half;
    pen.line(Point::new(left, rect.y + offset), Point::new(left, top));
    pen.line(Point::new(left, top), Point::new(right, top));
    pen.line(Point::new(right, top), Point::new(right, bottom));
    pen.line(
        Point::new(right, bottom),
        Point::new(rect.right() - offset, bottom),
    );
}
