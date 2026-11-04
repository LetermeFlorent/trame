use super::{GridPaint, Ink, boxes, style};
use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::term::{Cell, Flags};
use crate::text::{CellText, TextSystem};
use crate::theme::Rgba;

pub(super) struct RowPaint<'a> {
    pub(super) cells: &'a [Cell],
    pub(super) row: isize,
    pub(super) y: f32,
}

pub(super) fn draw_backgrounds(scene: &mut Scene, grid: &GridPaint, row: &RowPaint) {
    let GridPaint {
        theme,
        paint,
        metrics,
        ..
    } = grid;
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

pub(super) fn draw_glyphs(
    scene: &mut Scene,
    text: &mut TextSystem,
    grid: &GridPaint,
    row: &RowPaint,
) {
    let light = grid.metrics.underline_thickness.round().max(1.0);
    for (col, cell) in row.cells.iter().enumerate() {
        if cell.flags.contains(Flags::WIDE_SPACER) {
            continue;
        }
        let columns: u8 = if cell.flags.contains(Flags::WIDE) {
            2
        } else {
            1
        };
        let rect = cell_rect(grid, row, col, columns);
        let color = glyph_color(grid, row, col, cell);
        let ink = Ink { rect, color, light };
        if cell.flags.intersects(Flags::ANY_UNDERLINE) || cell.flags.contains(Flags::STRIKE) {
            style::draw_decorations(scene, cell.flags, &grid.metrics, ink);
        }
        let blank = matches!(cell.ch, ' ' | '\0') && cell.marks.is_empty();
        if blank || cell.flags.contains(Flags::HIDDEN) {
            continue;
        }
        if boxes::is_drawn(cell.ch) && boxes::draw(scene, cell.ch, ink) {
            continue;
        }
        let glyph = CellText {
            ch: cell.ch,
            marks: cell.marks,
            bold: cell.flags.contains(Flags::BOLD),
            italic: cell.flags.contains(Flags::ITALIC),
            columns,
        };
        text.draw_cell(scene, glyph, Point::new(rect.x, rect.y), color);
    }
}

fn cell_rect(grid: &GridPaint, row: &RowPaint, col: usize, columns: u8) -> Rect {
    let metrics = &grid.metrics;
    Rect::new(
        grid.paint.origin.x + col as f32 * metrics.width,
        row.y,
        metrics.width * f32::from(columns),
        metrics.height,
    )
}

fn glyph_color(grid: &GridPaint, row: &RowPaint, col: usize, cell: &Cell) -> Rgba {
    let row_index = usize::try_from(row.row).ok();
    let under_cursor = grid.cursor.is_some_and(|cursor| {
        row_index.is_some_and(|index| cursor.covers(index, col))
            && cursor.fills_cell(grid.paint.focused)
    });
    if under_cursor {
        grid.theme.terminal
    } else {
        style::colors(grid.theme, cell).0
    }
}
