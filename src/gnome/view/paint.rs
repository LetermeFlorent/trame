use gtk::graphene;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::term::graphics::Layer;
use trame::term::{Cell, Screen};
use trame::theme::{Rgba, Theme};
use trame::ui::terminal::{CursorPlace, colors};

use super::images::{covered, draw_tiles, prune};
use super::metrics::{self, Metrics};
use super::theme::fill;
use super::{TermView, cursor, text};

pub struct Paint<'a> {
    pub view: &'a TermView,
    pub snapshot: &'a gtk::Snapshot,
    pub theme: &'a Theme,
    pub metrics: &'a Metrics,
}

impl Paint<'_> {
    pub fn cell(&self, row: usize, col: usize, columns: usize) -> graphene::Rect {
        let Metrics { width, height, .. } = *self.metrics;
        graphene::Rect::new(
            col as f32 * width,
            row as f32 * height,
            columns as f32 * width,
            height,
        )
    }
}

pub fn draw(view: &TermView, snapshot: &gtk::Snapshot) {
    let imp = view.imp();
    let theme = imp.theme.borrow();
    let area = graphene::Rect::new(0.0, 0.0, view.width() as f32, view.height() as f32);
    fill(snapshot, theme.terminal, area);
    let session = imp.session.borrow();
    let Some(session) = session.as_ref() else {
        return;
    };
    let metrics = metrics::current(view);
    let paint = Paint {
        view,
        snapshot,
        theme: &theme,
        metrics: &metrics,
    };
    let terminal = session.terminal();
    screen(&paint, terminal.screen());
}

fn screen(paint: &Paint, screen: &Screen) {
    let grid = screen.grid();
    let rows: Vec<&[Cell]> = (0..grid.rows())
        .filter_map(|row| grid.view_row(row as isize))
        .map(|line| &line.cells()[..grid.cols().min(line.len())])
        .collect();
    let focused = paint.view.has_focus();
    let place = CursorPlace::locate(screen);
    let block = place.filter(|place| place.fills_cell(focused));
    let tiles = screen.image_tiles();
    draw_tiles(paint, &tiles, Layer::UnderBackground);
    for (row, cells) in rows.iter().enumerate() {
        backgrounds(paint, row, cells);
    }
    draw_tiles(paint, &tiles, Layer::UnderText);
    let block_area = block.map(|place| paint.cell(place.row, place.col, place.columns));
    if let Some(area) = block_area {
        fill(paint.snapshot, paint.theme.accent, area);
    }
    for (row, cells) in rows.iter().enumerate() {
        text::draw_row(paint, row, cells, grid.view_row(row as isize), block);
    }
    draw_tiles(paint, &tiles, Layer::OverText);
    if let Some(area) = block_area.filter(|area| covered(paint, &tiles, area)) {
        fill(paint.snapshot, paint.theme.accent, area);
    }
    if let Some(place) = place.filter(|_| block.is_none()) {
        cursor::outline(paint, place, focused);
    }
    prune(paint.view, &tiles);
}

fn backgrounds(paint: &Paint, row: usize, cells: &[Cell]) {
    let mut run: Option<(usize, usize, Rgba)> = None;
    let flush = |run: Option<(usize, usize, Rgba)>| {
        if let Some((start, end, color)) = run {
            fill(paint.snapshot, color, paint.cell(row, start, end - start));
        }
    };
    for (col, cell) in cells.iter().enumerate() {
        let (_, background) = colors(paint.theme, cell);
        let background = (background != paint.theme.terminal).then_some(background);
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
