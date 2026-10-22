use super::cursor::CursorPlace;
use super::{TerminalPaint, boxes, style};
use crate::color::Rgba;
use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::term::{Cell, Flags};
use crate::text::{CellMetrics, CellText, TextSystem};
use crate::theme::Theme;

pub(super) struct RowPaint<'a> {
    pub(super) cells: &'a [Cell],
    pub(super) row: isize,
    pub(super) y: f32,
}

pub(super) fn draw_backgrounds(
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

pub(super) fn draw_glyphs(
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
        let columns: u8 = if cell.flags.contains(Flags::WIDE) {
            2
        } else {
            1
        };
        let rect = Rect::new(
            paint.origin.x + col as f32 * metrics.width,
            row.y,
            metrics.width * f32::from(columns),
            metrics.height,
        );
        let (mut color, _) = style::colors(theme, cell);
        let row_index = usize::try_from(row.row).ok();
        if cursor.is_some_and(|cursor| {
            row_index.is_some_and(|index| cursor.covers(index, col))
                && cursor.fills_cell(paint.focused)
        }) {
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
