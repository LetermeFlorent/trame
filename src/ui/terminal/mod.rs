mod boxes;
mod cursor;
mod row;
mod style;

use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::term::Screen;
use crate::text::{CellMetrics, TextSystem};
use crate::theme::Rgba;
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

#[derive(Clone, Copy)]
struct Ink {
    rect: Rect,
    color: Rgba,
    light: f32,
}

struct GridPaint<'a> {
    theme: &'a Theme,
    paint: &'a TerminalPaint<'a>,
    metrics: CellMetrics,
    cursor: Option<CursorPlace>,
}

pub fn draw_terminal(
    scene: &mut Scene,
    text: &mut TextSystem,
    theme: &Theme,
    paint: &TerminalPaint,
) {
    let grid = GridPaint {
        theme,
        paint,
        metrics: text.cell_metrics(),
        cursor: CursorPlace::locate(paint.screen),
    };
    let metrics = grid.metrics;
    let origin = Point::new(
        paint.origin.x,
        paint.origin.y + (paint.scroll * metrics.height).round(),
    );
    let rows = visible_rows(paint, origin, &metrics);
    for row in &rows {
        draw_backgrounds(scene, &grid, row);
    }
    if let Some(cursor) = grid
        .cursor
        .filter(|cursor| cursor.fills_cell(paint.focused))
    {
        cursor.draw_fill(scene, theme, cursor.rect(origin, &metrics));
    }
    for row in &rows {
        draw_glyphs(scene, text, &grid, row);
    }
    if let Some(cursor) = grid.cursor {
        cursor.draw_outline(scene, &grid, cursor.rect(origin, &metrics));
    }
}

fn visible_rows<'a>(
    paint: &'a TerminalPaint,
    origin: Point,
    metrics: &CellMetrics,
) -> Vec<RowPaint<'a>> {
    let grid = paint.screen.grid();
    let cols = grid.cols();
    let first = (-paint.scroll).floor() as isize;
    let last = (grid.rows() as f32 - paint.scroll).ceil() as isize;
    (first..last)
        .filter_map(|row| {
            let line = grid.view_row(row)?;
            Some(RowPaint {
                cells: &line.cells()[..cols.min(line.len())],
                row,
                y: origin.y + row as f32 * metrics.height,
            })
        })
        .collect()
}
