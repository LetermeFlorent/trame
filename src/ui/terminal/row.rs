use super::links::LinkSpan;
use super::{GridPaint, Ink, boxes, style};
use crate::files::pictures::fit_size;
use crate::geometry::{Point, Rect};
use crate::render::{MAX_COLOR_ATLAS_SIZE, Scene};
use crate::term::grid::Row;
use crate::term::{Cell, Flags};
use crate::text::{CellText, TextSystem};
use crate::theme::Rgba;

pub(super) struct RowPaint<'a> {
    pub(super) cells: &'a [Cell],
    pub(super) line: &'a Row,
    pub(super) row: isize,
    pub(super) y: f32,
}

pub(super) fn draw_backgrounds(scene: &mut Scene, grid: &GridPaint, row: &RowPaint) {
    let (theme, paint, metrics) = (grid.theme, grid.paint, &grid.metrics);
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
    let links = row_links(grid.paint.links, row.row);
    for (col, cell) in row.cells.iter().enumerate() {
        if cell.flags.contains(Flags::WIDE_SPACER) || cell.flags.contains(Flags::IMAGE) {
            continue;
        }
        let columns = cell.columns();
        let rect = cell_rect(grid, row, col, columns);
        let color = glyph_color(grid, row, col, cell);
        let ink = Ink { rect, color, light };
        let linked = links.iter().any(|link| link.covers(col));
        decorate(scene, grid, cell.flags, linked, ink);
        let marks = row.line.marks(col);
        let blank = matches!(cell.ch, ' ' | '\0') && marks.is_empty();
        if blank || cell.flags.contains(Flags::HIDDEN) {
            continue;
        }
        if boxes::is_drawn(cell.ch) && boxes::draw(scene, cell.ch, ink) {
            continue;
        }
        let glyph = CellText {
            ch: cell.ch,
            marks,
            bold: cell.flags.contains(Flags::BOLD),
            italic: cell.flags.contains(Flags::ITALIC),
            columns,
        };
        text.draw_cell(scene, glyph, Point::new(rect.x, rect.y), color);
    }
}

fn decorate(scene: &mut Scene, grid: &GridPaint, mut flags: Flags, linked: bool, ink: Ink) {
    if linked && !flags.intersects(Flags::ANY_UNDERLINE) {
        flags.insert(Flags::UNDERLINE);
    }
    if flags.intersects(Flags::ANY_UNDERLINE) || flags.contains(Flags::STRIKE) {
        style::draw_decorations(scene, flags, &grid.metrics, ink);
    }
}

fn row_links(links: &[LinkSpan], row: isize) -> &[LinkSpan] {
    let start = links.partition_point(|link| link.row < row);
    let end = links.partition_point(|link| link.row <= row);
    &links[start..end]
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

pub(super) fn preview_fit(box_rect: Rect, src_w: u32, src_h: u32, limit: u32) -> (Rect, u32, u32) {
    let cap = limit.clamp(1, MAX_COLOR_ATLAS_SIZE - 1);
    let max_w = (box_rect.width.round().max(1.0) as u32).min(cap);
    let max_h = (box_rect.height.round().max(1.0) as u32).min(cap);
    let (width, height) = fit_size(src_w, src_h, max_w, max_h);
    let (w, h) = (width as f32, height as f32);
    let rect = Rect::new(
        box_rect.centered_x(w).round(),
        box_rect.centered_y(h).round(),
        w,
        h,
    );
    (rect, max_w, max_h)
}

pub fn preview_rect(box_rect: Rect, src_w: u32, src_h: u32) -> Rect {
    preview_fit(box_rect, src_w, src_h, MAX_COLOR_ATLAS_SIZE - 1).0
}
