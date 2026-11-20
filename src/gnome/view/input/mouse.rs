use std::time::{Duration, Instant};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::input::{MouseAction, MouseButton, MouseEvent, MouseModifiers, encode_mouse};
use trame::term::SelectionKind;

use crate::gnome::picture::rules::grid_cell;
use crate::gnome::view::{TermView, live, metrics};

const MULTI_CLICK: Duration = Duration::from_millis(500);
const CLICK_KINDS: [SelectionKind; 3] = [
    SelectionKind::Simple,
    SelectionKind::Word,
    SelectionKind::Line,
];

pub fn cell_under(view: &TermView, point: (f32, f32)) -> Option<(usize, usize)> {
    let m = metrics::current(view);
    live::with_terminal(view, |_, t, _| {
        grid_cell(
            point,
            (m.width, m.height),
            (t.screen().rows(), t.screen().cols()),
        )
    })?
}

pub fn count_click(view: &TermView, cell: (usize, usize)) -> usize {
    let now = Instant::now();
    let count = match view.imp().clicks.get() {
        Some((at, last, count)) if last == cell && now - at < MULTI_CLICK => {
            count % CLICK_KINDS.len() + 1
        }
        _ => 1,
    };
    view.imp().clicks.set(Some((now, cell, count)));
    count
}

pub fn start_selection(view: &TermView, cell: (usize, usize), extend: bool) {
    let count = count_click(view, cell);
    live::with_terminal(view, |_, terminal, _| {
        let screen = terminal.screen_mut();
        if extend && count == 1 && screen.selection().is_some() {
            screen.extend_selection(cell.0, cell.1);
        } else {
            screen.start_selection(CLICK_KINDS[count - 1], cell.0, cell.1);
        }
    });
    view.queue_draw();
}

pub fn send_mouse(
    view: &TermView,
    action: MouseAction,
    button: Option<MouseButton>,
    cell: (usize, usize),
) {
    let cur = view.imp().pointer.get();
    let event = MouseEvent {
        action,
        button,
        col: cell.1,
        row: cell.0,
        modifiers: MouseModifiers {
            shift: cur.shift,
            alt: cur.alt,
            ctrl: cur.ctrl,
        },
    };
    if let Some(Some(bytes)) =
        live::with_terminal(view, |_, t, _| encode_mouse(&event, t.screen().modes()))
    {
        live::write(view, bytes);
    }
}
