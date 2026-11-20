mod bg;
mod cursor;
mod hover;
mod pictures;
pub mod strips;
mod style;
mod text;

use gtk::graphene;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::geometry::Rect;
use trame::term::Screen;
use trame::theme::Theme;
use trame::ui::terminal::links::LinkSpan;
use trame::ui::terminal::{CursorPlace, drawn_rows};

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
        self.cell_y(row as f32 * self.metrics.height, col, columns)
    }

    pub fn cell_y(&self, y: f32, col: usize, columns: usize) -> graphene::Rect {
        graphene::Rect::new(
            col as f32 * self.metrics.width,
            y,
            columns as f32 * self.metrics.width,
            self.metrics.height,
        )
    }
}

pub fn draw(view: &TermView, snapshot: &gtk::Snapshot) {
    let imp = view.imp();
    let theme = imp.theme.borrow();
    let bounds = Rect::new(0.0, 0.0, view.width() as f32, view.height() as f32);
    fill(snapshot, theme.terminal, to_graphene(bounds));
    let metrics = metrics::current(view);
    let scroll = imp.scroll.get();
    let detect_links = imp.detect_links.get();
    live::with_terminal(view, |id, terminal, cwd| {
        let range = drawn_rows(terminal.screen(), scroll);
        let mut link_cache = imp.links.borrow_mut();
        let links: &[LinkSpan] = if detect_links {
            link_cache.spans(id, terminal, range.clone(), cwd)
        } else {
            &[]
        };
        let paint = Paint {
            view,
            snapshot,
            theme: &theme,
            metrics: &metrics,
            links,
            session_id: id.0,
            bounds,
        };
        screen(
            &paint,
            terminal.screen(),
            range,
            (scroll * metrics.height).round(),
        );
    });
    drop(theme);
    hover::draw_card(view, snapshot, &imp.theme.borrow(), bounds);
}

fn screen(paint: &Paint, screen: &Screen, range: std::ops::Range<isize>, scroll_y: f32) {
    let grid = screen.grid();
    let rows: Vec<RowCells> = range
        .filter_map(|row| {
            let line = grid.view_row(row)?;
            Some(RowCells {
                row,
                y: row as f32 * paint.metrics.height + scroll_y,
                cells: &line.cells()[..grid.cols().min(line.len())],
            })
        })
        .collect();
    let focused = paint.view.has_focus();
    let place = CursorPlace::locate(screen);
    let block = place.filter(|place| place.fills_cell(focused));
    for item in &rows {
        bg::backgrounds(paint, screen, item);
    }
    if let Some(place) = block {
        let area = paint.cell_y(
            place.row as f32 * paint.metrics.height + scroll_y,
            place.col,
            place.columns,
        );
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
