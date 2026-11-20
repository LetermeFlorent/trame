mod cursor;
mod hover;
mod pictures;
pub mod strips;
mod text;

use gtk::graphene;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::geometry::Rect;
use trame::term::{Cell, Screen};
use trame::theme::{Rgba, Theme};
use trame::ui::terminal::links::LinkSpan;
use trame::ui::terminal::{CursorPlace, colors};

use super::TermView;
use super::live;
use super::metrics::{self, Metrics};
use super::theme::fill;
use strips::{RowCells, to_graphene};

pub struct Paint<'a> {
    pub view: &'a TermView,
    pub snapshot: &'a gtk::Snapshot,
    pub theme: &'a Theme,
    pub metrics: &'a Metrics,
    pub links: &'a [LinkSpan],
    pub session_id: u64,
    pub bounds: Rect,
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
    let bounds = Rect::new(0.0, 0.0, view.width() as f32, view.height() as f32);
    fill(snapshot, theme.terminal, to_graphene(bounds));
    let cwd = live::session_cwd(view);
    let session_guard = imp.session.borrow();
    let Some(session) = session_guard.as_ref() else {
        return;
    };
    let metrics = metrics::current(view);
    let terminal = session.terminal();
    let grid_rows = terminal.screen().rows() as isize;
    let mut link_cache = imp.links.borrow_mut();
    let links = link_cache.spans(session.id(), &terminal, 0..grid_rows, cwd.as_deref());
    let paint = Paint {
        view,
        snapshot,
        theme: &theme,
        metrics: &metrics,
        links,
        session_id: session.id().0,
        bounds,
    };
    screen(&paint, terminal.screen());
    drop(link_cache);
    drop(terminal);
    drop(session_guard);
    hover::draw_card(view, snapshot, &theme, bounds);
}

fn screen(paint: &Paint, screen: &Screen) {
    let grid = screen.grid();
    let rows: Vec<RowCells> = (0..grid.rows())
        .filter_map(|row| {
            let line = grid.view_row(row as isize)?;
            Some(RowCells {
                row: row as isize,
                y: row as f32 * paint.metrics.height,
                cells: &line.cells()[..grid.cols().min(line.len())],
            })
        })
        .collect();
    let focused = paint.view.has_focus();
    let place = CursorPlace::locate(screen);
    let block = place.filter(|place| place.fills_cell(focused));
    for item in &rows {
        backgrounds(paint, item.row as usize, item.cells);
    }
    if let Some(place) = block {
        let area = paint.cell(place.row, place.col, place.columns);
        fill(paint.snapshot, paint.theme.accent, area);
    }
    for item in &rows {
        let line = grid.view_row(item.row);
        text::draw_row(paint, item, line, block);
    }
    pictures::draw_images(paint, screen, &rows);
    if let Some(place) = place.filter(|_| block.is_none()) {
        cursor::outline(paint, place, focused);
    }
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
