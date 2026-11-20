use gtk::gdk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::input::MouseAction;

use super::hits::inspect_hover;
use super::mouse::{cell_under, send_mouse};
use super::pointer::{apply_cursor, update_pointer};
use crate::gnome::view::{TermView, hover, live};

const CURSOR_POINTER: &str = "pointer";
const CURSOR_TEXT: &str = "text";

pub fn motion_controller(view: &TermView) -> gtk::EventControllerMotion {
    let motion = gtk::EventControllerMotion::new();
    let entered = view.downgrade();
    motion.connect_enter(move |ctrl, x, y| {
        if let Some(view) = entered.upgrade() {
            update_pointer(
                &view,
                Some((x as f32, y as f32)),
                ctrl.current_event_state(),
            );
            refresh_hover(&view);
        }
    });
    let moved = view.downgrade();
    motion.connect_motion(move |ctrl, x, y| {
        if let Some(view) = moved.upgrade() {
            on_motion(&view, x as f32, y as f32, ctrl.current_event_state());
        }
    });
    let left = view.downgrade();
    motion.connect_leave(move |_| {
        if let Some(view) = left.upgrade() {
            leave_view(&view);
        }
    });
    motion
}

fn on_motion(view: &TermView, x: f32, y: f32, state: gdk::ModifierType) {
    update_pointer(view, Some((x, y)), state);
    let mut cur = view.imp().pointer.get();
    if let Some(cell) = cell_under(view, (x, y))
        && cur.last_cell != Some(cell)
    {
        if cur.dragging {
            cur.last_cell = Some(cell);
            view.imp().pointer.set(cur);
            live::with_terminal(view, |_, t, _| {
                t.screen_mut().extend_selection(cell.0, cell.1)
            });
            view.queue_draw();
            return;
        }
        if let Some(btn) = cur.reported_button {
            cur.last_cell = Some(cell);
            view.imp().pointer.set(cur);
            send_mouse(view, MouseAction::Motion, Some(btn), cell);
            return;
        }
    }
    refresh_hover(view);
}

fn leave_view(view: &TermView) {
    let mut cur = view.imp().pointer.get();
    cur.position = None;
    view.imp().pointer.set(cur);
    hover::clear_hover(view);
    apply_cursor(view, CURSOR_TEXT);
}

pub fn sync_key_mods(view: &TermView, keyval: gdk::Key, state: gdk::ModifierType, down: bool) {
    let mut cur = view.imp().pointer.get();
    let is_ctrl = matches!(keyval, gdk::Key::Control_L | gdk::Key::Control_R);
    let is_shift = matches!(keyval, gdk::Key::Shift_L | gdk::Key::Shift_R);
    let is_alt = matches!(keyval, gdk::Key::Alt_L | gdk::Key::Alt_R);
    cur.ctrl = if is_ctrl {
        down
    } else {
        state.contains(gdk::ModifierType::CONTROL_MASK)
    };
    cur.shift = if is_shift {
        down
    } else {
        state.contains(gdk::ModifierType::SHIFT_MASK)
    };
    cur.alt = if is_alt {
        down
    } else {
        state.contains(gdk::ModifierType::ALT_MASK)
    };
    view.imp().pointer.set(cur);
    refresh_hover(view);
}

pub fn clear_mods(view: &TermView) {
    let mut cur = view.imp().pointer.get();
    cur.ctrl = false;
    cur.shift = false;
    cur.alt = false;
    view.imp().pointer.set(cur);
    apply_cursor(view, CURSOR_TEXT);
}

pub fn refresh_hover(view: &TermView) {
    let cur = view.imp().pointer.get();
    if view.picture_open() || cur.position.is_none() {
        hover::clear_hover(view);
        apply_cursor(view, CURSOR_TEXT);
        return;
    }
    let (clickable, preview) =
        inspect_hover(view, cur.position.unwrap_or_default(), cur.ctrl, cur.shift);
    apply_cursor(
        view,
        if clickable {
            CURSOR_POINTER
        } else {
            CURSOR_TEXT
        },
    );
    match preview {
        Some((path, anchor)) => hover::set_hover(view, path, anchor),
        None => hover::clear_hover(view),
    }
}
