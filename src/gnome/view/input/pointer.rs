use gtk::gdk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::input::{MouseAction, MouseButton};

use super::hits::activate_press;
use super::motion::motion_controller;
use super::mouse::{cell_under, send_mouse, start_selection};
use crate::gnome::picture::rules::reports_mouse;
use crate::gnome::view::{TermView, live};

pub fn attach(view: &TermView) {
    let click = gtk::GestureClick::new();
    click.set_button(0);
    let pressed = view.downgrade();
    click.connect_pressed(move |g, _, x, y| {
        if let Some(view) = pressed.upgrade() {
            on_press(&view, g, x as f32, y as f32);
        }
    });
    let released = view.downgrade();
    click.connect_released(move |g, _, x, y| {
        if let Some(view) = released.upgrade() {
            on_release(&view, g, x as f32, y as f32);
        }
    });
    view.add_controller(click);
    view.add_controller(motion_controller(view));
}

fn on_press(view: &TermView, gesture: &gtk::GestureClick, x: f32, y: f32) {
    view.grab_focus();
    update_pointer(view, Some((x, y)), gesture.current_event_state());
    if view.picture_open() {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        return;
    }
    let btn = gdk_button(gesture.current_button());
    if btn == Some(MouseButton::Left) && activate_press(view) {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        return;
    }
    let cur = view.imp().pointer.get();
    let reports = live::with_terminal(view, |_, t, _| reports_mouse(t.screen().modes(), cur.shift))
        == Some(true);
    let Some(cell) = cell_under(view, (x, y)) else {
        return;
    };
    if reports && let Some(btn) = btn {
        update_drag(view, false, Some(btn), Some(cell));
        send_mouse(view, MouseAction::Press, Some(btn), cell);
        gesture.set_state(gtk::EventSequenceState::Claimed);
    } else if btn == Some(MouseButton::Middle) {
        if let Some(cb) = view.imp().middle_click.borrow().as_ref() {
            cb();
        }
        gesture.set_state(gtk::EventSequenceState::Claimed);
    } else if btn == Some(MouseButton::Left) {
        update_drag(view, true, None, Some(cell));
        start_selection(view, cell, cur.shift);
        gesture.set_state(gtk::EventSequenceState::Claimed);
    }
}

fn on_release(view: &TermView, gesture: &gtk::GestureClick, x: f32, y: f32) {
    update_pointer(view, Some((x, y)), gesture.current_event_state());
    let mut cur = view.imp().pointer.get();
    if let Some(btn) = cur.reported_button.take() {
        cur.last_cell = None;
        view.imp().pointer.set(cur);
        if let Some(cell) = cell_under(view, (x, y)) {
            send_mouse(view, MouseAction::Release, Some(btn), cell);
        }
    } else if cur.dragging {
        cur.dragging = false;
        cur.last_cell = None;
        view.imp().pointer.set(cur);
        let text = live::with_terminal(view, |_, t, _| t.screen().selected_text()).flatten();
        if let (Some(text), Some(cb)) = (text, view.imp().primary_selection.borrow().as_ref()) {
            cb(text);
        }
    }
}

fn gdk_button(button: u32) -> Option<MouseButton> {
    match button {
        gdk::BUTTON_PRIMARY => Some(MouseButton::Left),
        gdk::BUTTON_MIDDLE => Some(MouseButton::Middle),
        gdk::BUTTON_SECONDARY => Some(MouseButton::Right),
        _ => None,
    }
}

fn update_drag(
    view: &TermView,
    dragging: bool,
    reported: Option<MouseButton>,
    cell: Option<(usize, usize)>,
) {
    let mut cur = view.imp().pointer.get();
    cur.dragging = dragging;
    cur.reported_button = reported;
    cur.last_cell = cell;
    view.imp().pointer.set(cur);
}

pub(super) fn update_pointer(view: &TermView, pos: Option<(f32, f32)>, state: gdk::ModifierType) {
    let mut cur = view.imp().pointer.get();
    cur.position = pos;
    cur.ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
    cur.shift = state.contains(gdk::ModifierType::SHIFT_MASK);
    cur.alt = state.contains(gdk::ModifierType::ALT_MASK);
    view.imp().pointer.set(cur);
}

pub(super) fn apply_cursor(view: &TermView, name: &'static str) {
    let mut cur = view.imp().pointer.get();
    if cur.cursor != name {
        cur.cursor = name;
        view.imp().pointer.set(cur);
        view.set_cursor_from_name(Some(name));
    }
}
