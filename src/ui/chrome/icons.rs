use crate::color::Rgba;
use crate::geometry::{Point, Rect};
use crate::render::Scene;

const FRAME_INSET: f32 = 0.07;
const FRAME_RADIUS: f32 = 0.18;
const SIDEBAR_SPLIT: f32 = 0.38;
const PANEL_ALPHA: f32 = 0.35;
const PLUS_REACH: f32 = 0.36;
const CROSS_REACH: f32 = 0.28;
const DASH_REACH: f32 = 0.36;
const RESTORE_OFFSET: f32 = 0.2;
const GRID_GAP: f32 = 0.14;
const GRID_RADIUS: f32 = 0.12;
const CHEVRON_LEFT: f32 = 0.2;
const CHEVRON_TIP: f32 = 0.46;
const CHEVRON_TOP: f32 = 0.28;
const CHEVRON_BOTTOM: f32 = 0.72;
const CURSOR_LINE_Y: f32 = 0.74;
const CURSOR_LINE_LEFT: f32 = 0.56;
const CURSOR_LINE_RIGHT: f32 = 0.84;

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
    Instance,
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
        Icon::Instance => grid(pen, rect, size),
    }
}

fn icon_frame(rect: Rect, size: f32) -> Rect {
    rect.inset((size * FRAME_INSET).round()).snapped()
}

fn sidebar(pen: &mut Pen, rect: Rect, size: f32, open: bool) {
    let frame = icon_frame(rect, size);
    let split = pen.crisp(frame.x + frame.width * SIDEBAR_SPLIT);
    if open {
        let panel = Rect::from_edges(frame.x, frame.y, split, frame.bottom());
        let radius = size * FRAME_RADIUS;
        pen.scene.fill(
            panel,
            pen.color.with_alpha(pen.color.a * PANEL_ALPHA),
            radius,
        );
    }
    pen.frame(frame, size * FRAME_RADIUS);
    pen.line(
        Point::new(split, frame.y),
        Point::new(split, frame.bottom()),
    );
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

fn split(pen: &mut Pen, rect: Rect, size: f32, vertical: bool) {
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

fn terminal(pen: &mut Pen, rect: Rect, size: f32) {
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

fn grid(pen: &mut Pen, rect: Rect, size: f32) {
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
