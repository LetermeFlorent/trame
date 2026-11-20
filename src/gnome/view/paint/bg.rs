use trame::term::{Cell, Screen};
use trame::theme::{Rgba, Theme};
use trame::ui::terminal::colors;

use super::Paint;
use super::strips::RowCells;
use crate::gnome::view::theme::fill;

pub(super) fn backgrounds(paint: &Paint, screen: &Screen, item: &RowCells) {
    let mut run: Option<(usize, usize, Rgba)> = None;
    let flush = |run: Option<(usize, usize, Rgba)>| {
        if let Some((start, end, color)) = run {
            fill(
                paint.snapshot,
                color,
                paint.cell_y(item.y, start, end - start),
            );
        }
    };
    for (col, cell) in item.cells.iter().enumerate() {
        let background = cell_bg(paint.theme, screen, item.row, col, cell);
        match (run, background) {
            (Some((start, end, color)), Some(next)) if end == col && color == next => {
                run = Some((start, col + 1, color));
            }
            (_, next) => {
                flush(run);
                run = next.map(|color| (col, col + 1, color));
            }
        }
    }
    flush(run);
}

fn cell_bg(theme: &Theme, screen: &Screen, row: isize, col: usize, cell: &Cell) -> Option<Rgba> {
    if screen.grid().is_selected_in_view(row, col) {
        return Some(theme.selection);
    }
    let (_, background) = colors(theme, cell);
    (background != theme.terminal).then_some(background)
}
