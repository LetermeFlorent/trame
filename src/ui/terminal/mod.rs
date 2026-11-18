mod boxes;
mod copy;
mod cursor;
mod images;
pub mod links;
mod row;
mod style;

use std::ops::Range;

use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::term::Screen;
use crate::term::graphics::Layer;
use crate::text::{CellMetrics, TextSystem};
use crate::theme::Rgba;
use crate::theme::Theme;

pub use copy::ScreenCopy;
pub use cursor::CursorPlace;
pub use images::{draw_tiles, tile_rect};
use links::LinkSpan;
use row::{RowPaint, draw_backgrounds, draw_glyphs};
pub use style::colors;

pub struct TerminalPaint<'a> {
    pub screen: &'a ScreenCopy,
    pub origin: Point,
    pub focused: bool,
    pub scroll: f32,
    pub selected: &'a dyn Fn(isize, usize) -> bool,
    pub links: &'a [LinkSpan],
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
        cursor: paint.screen.cursor,
    };
    let metrics = grid.metrics;
    let origin = Point::new(
        paint.origin.x,
        paint.origin.y + (paint.scroll * metrics.height).round(),
    );
    let rows = visible_rows(paint, origin, &metrics);
    let tiles = &paint.screen.tiles;
    draw_tiles(scene, &metrics, origin, tiles, Layer::UnderBackground);
    for row in &rows {
        draw_backgrounds(scene, &grid, row);
    }
    draw_tiles(scene, &metrics, origin, tiles, Layer::UnderText);
    draw_cursor_fill(scene, &grid, origin);
    for row in &rows {
        draw_glyphs(scene, text, &grid, row);
    }
    draw_tiles(scene, &metrics, origin, tiles, Layer::OverText);
    if !tiles.is_empty() {
        images::cover_tiles(scene, &grid, &rows, origin);
    }
    if let Some(cursor) = grid.cursor {
        cursor.draw_outline(scene, &grid, cursor.rect(origin, &metrics));
    }
}

fn draw_cursor_fill(scene: &mut Scene, grid: &GridPaint, origin: Point) {
    let focused = grid.paint.focused;
    if let Some(cursor) = grid.cursor.filter(|cursor| cursor.fills_cell(focused)) {
        cursor.draw_fill(scene, grid.theme, cursor.rect(origin, &grid.metrics));
    }
}

fn visible_rows<'a>(
    paint: &'a TerminalPaint,
    origin: Point,
    metrics: &CellMetrics,
) -> Vec<RowPaint<'a>> {
    let cols = paint.screen.cols;
    let rows = paint.screen.rows.iter();
    rows.map(|(row, line)| RowPaint {
        cells: &line.cells()[..cols.min(line.len())],
        line,
        row: *row,
        y: origin.y + *row as f32 * metrics.height,
    })
    .collect()
}

pub fn drawn_rows(screen: &Screen, scroll: f32) -> Range<isize> {
    let first = (-scroll).floor() as isize;
    let last = (screen.grid().rows() as f32 - scroll).ceil() as isize;
    first..last
}
