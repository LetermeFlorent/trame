mod hits;
mod keys;
mod pointer;

pub use pointer::refresh_hover;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use trame::input::paste_bytes;
use trame::term::Modes;

use super::{TermView, hover, live};

const PASTE_SHORTCUT: gdk::ModifierType =
    gdk::ModifierType::CONTROL_MASK.union(gdk::ModifierType::SHIFT_MASK);

pub fn attach(view: &TermView) {
    let im = gtk::IMMulticontext::new();
    im.set_client_widget(Some(view));
    let typed = view.downgrade();
    im.connect_commit(move |_, text| {
        if let Some(view) = typed.upgrade()
            && !view.picture_open()
        {
            live::write(&view, text.as_bytes().to_vec());
        }
    });
    view.add_controller(key_controller(view, &im));
    view.add_controller(focus_watch(view, &im));
    pointer::attach(view);
}

fn key_controller(view: &TermView, im: &gtk::IMMulticontext) -> gtk::EventControllerKey {
    let keys = gtk::EventControllerKey::new();
    keys.set_im_context(Some(im));
    let pressed = view.downgrade();
    keys.connect_key_pressed(move |_, keyval, _, state| {
        let handled = pressed
            .upgrade()
            .is_some_and(|view| key_pressed(&view, keyval, state));
        glib::Propagation::from(handled)
    });
    let released = view.downgrade();
    keys.connect_key_released(move |_, keyval, _, state| {
        if let Some(view) = released.upgrade() {
            pointer::sync_key_mods(&view, keyval, state, false);
        }
    });
    keys
}

fn key_pressed(view: &TermView, keyval: gdk::Key, state: gdk::ModifierType) -> bool {
    pointer::sync_key_mods(view, keyval, state, true);
    if view.picture_open() {
        return true;
    }
    let state = state & gtk::accelerator_get_default_mod_mask();
    if state == PASTE_SHORTCUT && keyval.to_lower() == gdk::Key::v {
        paste(view);
        return true;
    }
    match keys::bytes(keyval, state, modes(view)) {
        Some(bytes) => {
            live::write(view, bytes);
            true
        }
        None => false,
    }
}

pub(super) fn modes(view: &TermView) -> Modes {
    view.imp()
        .session
        .borrow()
        .as_ref()
        .map(|session| session.terminal().screen().modes())
        .unwrap_or_default()
}

fn paste(view: &TermView) {
    let target = view.downgrade();
    view.clipboard()
        .read_text_async(None::<&gio::Cancellable>, move |text| {
            let (Some(view), Ok(Some(text))) = (target.upgrade(), text) else {
                return;
            };
            let bytes = paste_bytes(&text, modes(&view));
            live::write(&view, bytes);
        });
}

fn focus_watch(view: &TermView, im: &gtk::IMMulticontext) -> gtk::EventControllerFocus {
    let focus = gtk::EventControllerFocus::new();
    let entered = im.clone();
    focus.connect_enter(move |focus| {
        entered.focus_in();
        if let Some(widget) = focus.widget() {
            widget.queue_draw();
        }
    });
    let left_im = im.clone();
    let left_view = view.downgrade();
    focus.connect_leave(move |focus| {
        left_im.focus_out();
        if let Some(view) = left_view.upgrade() {
            pointer::clear_mods(&view);
            hover::clear_hover(&view);
        }
        if let Some(widget) = focus.widget() {
            widget.queue_draw();
        }
    });
    focus
}
