mod boxes;
mod cursor;
mod row;
mod style;

use crate::geometry::Point;
use crate::render::Scene;
use crate::term::Screen;
use crate::text::TextSystem;
use crate::theme::Theme;

pub use cursor::CursorPlace;
use row::{RowPaint, draw_backgrounds, draw_glyphs};

pub struct TerminalPaint<'a> {
    pub screen: &'a Screen,
    pub origin: Point,
    pub focused: bool,
    pub scroll: f32,
    pub selected: &'a dyn Fn(isize, usize) -> bool,
}

pub fn draw_terminal(
    scene: &mut Scene,
    text: &mut TextSystem,
    theme: &Theme,
    paint: &TerminalPaint,
) {
    let metrics = text.cell_metrics();
    let grid = paint.screen.grid();
    let cols = grid.cols();
    let cursor = CursorPlace::locate(paint.screen);
    let origin = Point::new(
        paint.origin.x,
        paint.origin.y + (paint.scroll * metrics.height).round(),
    );
    let first = (-paint.scroll).floor() as isize;
    let last = (grid.rows() as f32 - paint.scroll).ceil() as isize;
    let rows: Vec<RowPaint> = (first..last)
        .filter_map(|row| {
            let line = grid.view_row(row)?;
            Some(RowPaint {
                cells: &line.cells()[..cols.min(line.len())],
                row,
                y: origin.y + row as f32 * metrics.height,
            })
        })
        .collect();
    for row in &rows {
        draw_backgrounds(scene, theme, paint, row, &metrics);
    }
    if let Some(cursor) = cursor.filter(|cursor| cursor.fills_cell(paint.focused)) {
        cursor.draw_fill(scene, theme, cursor.rect(origin, &metrics));
    }
    for row in &rows {
        draw_glyphs(scene, text, theme, paint, row, cursor, &metrics);
    }
    if let Some(cursor) = cursor {
        let rect = cursor.rect(origin, &metrics);
        cursor.draw_outline(scene, theme, rect, &metrics, paint.focused);
    }
}
