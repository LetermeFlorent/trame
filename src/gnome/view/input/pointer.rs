use gtk::gdk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::hits::{activate_press, inspect_hover};
use crate::gnome::view::{TermView, hover};

const CURSOR_POINTER: &str = "pointer";
const CURSOR_TEXT: &str = "text";

pub fn attach(view: &TermView) {
    let click = gtk::GestureClick::new();
    let clicked = view.downgrade();
    click.connect_pressed(move |gesture, _, x, y| {
        let Some(view) = clicked.upgrade() else {
            return;
        };
        view.grab_focus();
        let mods = gesture.current_event_state();
        update_pointer(&view, Some((x as f32, y as f32)), mods);
        if gesture.current_button() == gdk::BUTTON_PRIMARY && activate_press(&view) {
            gesture.set_state(gtk::EventSequenceState::Claimed);
        }
    });
    view.add_controller(click);
    view.add_controller(motion_controller(view));
}

fn motion_controller(view: &TermView) -> gtk::EventControllerMotion {
    let motion = gtk::EventControllerMotion::new();
    let entered = view.downgrade();
    motion.connect_enter(move |ctrl, x, y| {
        if let Some(view) = entered.upgrade() {
            let mods = ctrl.current_event_state();
            update_pointer(&view, Some((x as f32, y as f32)), mods);
            refresh_hover(&view);
        }
    });
    let moved = view.downgrade();
    motion.connect_motion(move |ctrl, x, y| {
        if let Some(view) = moved.upgrade() {
            let mods = ctrl.current_event_state();
            update_pointer(&view, Some((x as f32, y as f32)), mods);
            refresh_hover(&view);
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

fn leave_view(view: &TermView) {
    let mut state = view.imp().pointer.get();
    state.position = None;
    view.imp().pointer.set(state);
    hover::clear_hover(view);
    apply_cursor(view, CURSOR_TEXT);
}

pub fn sync_key_mods(view: &TermView, keyval: gdk::Key, state: gdk::ModifierType, down: bool) {
    let mut cur = view.imp().pointer.get();
    let is_ctrl = matches!(keyval, gdk::Key::Control_L | gdk::Key::Control_R);
    let is_shift = matches!(keyval, gdk::Key::Shift_L | gdk::Key::Shift_R);
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
    view.imp().pointer.set(cur);
    refresh_hover(view);
}

pub fn clear_mods(view: &TermView) {
    let mut cur = view.imp().pointer.get();
    cur.ctrl = false;
    cur.shift = false;
    view.imp().pointer.set(cur);
    apply_cursor(view, CURSOR_TEXT);
}

fn update_pointer(view: &TermView, pos: Option<(f32, f32)>, state: gdk::ModifierType) {
    let mut cur = view.imp().pointer.get();
    cur.position = pos;
    cur.ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
    cur.shift = state.contains(gdk::ModifierType::SHIFT_MASK);
    view.imp().pointer.set(cur);
}

pub fn refresh_hover(view: &TermView) {
    let cur = view.imp().pointer.get();
    if view.picture_open() || cur.position.is_none() {
        hover::clear_hover(view);
        apply_cursor(view, CURSOR_TEXT);
        return;
    }
    let point = cur.position.unwrap_or_default();
    let (clickable, preview) = inspect_hover(view, point, cur.ctrl, cur.shift);
    let cursor = if clickable {
        CURSOR_POINTER
    } else {
        CURSOR_TEXT
    };
    apply_cursor(view, cursor);
    match preview {
        Some((path, anchor)) => hover::set_hover(view, path, anchor),
        None => hover::clear_hover(view),
    }
}

fn apply_cursor(view: &TermView, name: &'static str) {
    let mut cur = view.imp().pointer.get();
    if cur.cursor != name {
        cur.cursor = name;
        view.imp().pointer.set(cur);
        view.set_cursor_from_name(Some(name));
    }
}
