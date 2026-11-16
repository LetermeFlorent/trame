use gtk::graphene;
use trame::term::CursorShape;
use trame::ui::terminal::CursorPlace;

use super::paint::Paint;
use super::theme::fill;

const BAR_RATIO: f32 = 1.0 / 7.0;
const HOLLOW_STROKE: f32 = 1.0;

pub fn outline(paint: &Paint, place: CursorPlace, focused: bool) {
    let cell = paint.cell(place.row, place.col, place.columns);
    let (x, y, width, height) = (cell.x(), cell.y(), cell.width(), cell.height());
    let edges = match place.shape {
        CursorShape::Bar => vec![(x, y, thickness(width), height)],
        CursorShape::Underline => {
            let line = thickness(height);
            vec![(x, y + height - line, width, line)]
        }
        CursorShape::Block if !focused => hollow(x, y, width, height),
        CursorShape::Block => Vec::new(),
    };
    for (x, y, width, height) in edges {
        fill(
            paint.snapshot,
            paint.theme.accent,
            graphene::Rect::new(x, y, width, height),
        );
    }
}

fn thickness(size: f32) -> f32 {
    (size * BAR_RATIO).round().max(1.0)
}

fn hollow(x: f32, y: f32, width: f32, height: f32) -> Vec<(f32, f32, f32, f32)> {
    let stroke = HOLLOW_STROKE;
    vec![
        (x, y, width, stroke),
        (x, y + height - stroke, width, stroke),
        (x, y, stroke, height),
        (x + width - stroke, y, stroke, height),
    ]
}
