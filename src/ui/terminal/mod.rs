mod boxes;
mod cursor;
mod style;

use crate::color::Rgba;
use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::term::{Cell, Flags, Screen};
use crate::text::{CellMetrics, CellText, TextSystem};
use crate::theme::Theme;

pub use cursor::CursorPlace;

pub struct TerminalPaint<'a> {
    pub screen: &'a Screen,
    pub origin: Point,
    pub focused: bool,
    pub selected: &'a dyn Fn(usize, usize) -> bool,
}

struct RowPaint<'a> {
    cells: &'a [Cell],
    row: usize,
    y: f32,
}

pub fn draw_terminal(scene: &mut Scene, text: &mut TextSystem, theme: &Theme, paint: &TerminalPaint) {
    let metrics = text.cell_metrics();
    let grid = paint.screen.grid();
    let cols = grid.cols();
    let cursor = CursorPlace::locate(paint.screen);
    let rows: Vec<RowPaint> = (0..grid.rows())
        .map(|row| RowPaint {
            cells: &grid.visible_row(row).cells()[..cols.min(grid.visible_row(row).len())],
            row,
            y: paint.origin.y + row as f32 * metrics.height,
        })
        .collect();
    for row in &rows {
        draw_backgrounds(scene, theme, paint, row, &metrics);
    }
    if let Some(cursor) = cursor.filter(|cursor| cursor.fills_cell(paint.focused)) {
        cursor.draw_fill(scene, theme, cursor.rect(paint.origin, &metrics));
    }
    for row in &rows {
        draw_glyphs(scene, text, theme, paint, row, cursor, &metrics);
    }
    if let Some(cursor) = cursor {
        let rect = cursor.rect(paint.origin, &metrics);
        cursor.draw_outline(scene, theme, rect, &metrics, paint.focused);
    }
}

fn draw_backgrounds(
    scene: &mut Scene,
    theme: &Theme,
    paint: &TerminalPaint,
    row: &RowPaint,
    metrics: &CellMetrics,
) {
    let mut run: Option<(usize, usize, Rgba)> = None;
    let flush = |scene: &mut Scene, run: Option<(usize, usize, Rgba)>| {
        if let Some((start, end, color)) = run {
            let x = paint.origin.x + start as f32 * metrics.width;
            let width = (end - start) as f32 * metrics.width;
            scene.fill(Rect::new(x, row.y, width, metrics.height), color, 0.0);
        }
    };
    for (col, cell) in row.cells.iter().enumerate() {
        let background = if (paint.selected)(row.row, col) {
            Some(theme.selection)
        } else {
            let (_, background) = style::colors(theme, cell);
            (background != theme.terminal).then_some(background)
        };
        match (run, background) {
            (Some((start, end, color)), Some(next)) if end == col && color == next => {
                run = Some((start, col + 1, color));
            }
            (_, next) => {
                flush(scene, run);
                run = next.map(|color| (col, col + 1, color));
            }
        }
    }
    flush(scene, run);
}

fn draw_glyphs(
    scene: &mut Scene,
    text: &mut TextSystem,
    theme: &Theme,
    paint: &TerminalPaint,
    row: &RowPaint,
    cursor: Option<CursorPlace>,
    metrics: &CellMetrics,
) {
    let light = metrics.underline_thickness.round().max(1.0);
    for (col, cell) in row.cells.iter().enumerate() {
        if cell.flags.contains(Flags::WIDE_SPACER) {
            continue;
        }
        let columns: u8 = if cell.flags.contains(Flags::WIDE) { 2 } else { 1 };
        let rect = Rect::new(
            paint.origin.x + col as f32 * metrics.width,
            row.y,
            metrics.width * f32::from(columns),
            metrics.height,
        );
        let (mut color, _) = style::colors(theme, cell);
        if cursor.is_some_and(|cursor| cursor.covers(row.row, col) && cursor.fills_cell(paint.focused)) {
            color = theme.terminal;
        }
        if cell.flags.intersects(Flags::ANY_UNDERLINE) || cell.flags.contains(Flags::STRIKE) {
            style::draw_decorations(scene, cell.flags, rect, metrics, color);
        }
        if cell.ch == ' ' || cell.ch == '\0' || cell.flags.contains(Flags::HIDDEN) {
            continue;
        }
        if boxes::is_drawn(cell.ch) && boxes::draw(scene, cell.ch, rect, color, light) {
            continue;
        }
        let glyph = CellText {
            ch: cell.ch,
            bold: cell.flags.contains(Flags::BOLD),
            italic: cell.flags.contains(Flags::ITALIC),
            columns,
        };
        text.draw_cell(scene, glyph, Point::new(rect.x, rect.y), color);
    }
}
