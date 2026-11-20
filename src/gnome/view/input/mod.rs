mod hits;
mod keys;
mod motion;
mod mouse;
mod pointer;

pub use motion::refresh_hover;

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
    keys.connect_key_pressed(move |ctrl, keyval, _, state| {
        if ctrl
            .current_event()
            .is_none_or(|_| pressed.upgrade().is_none_or(|v| v.picture_open()))
        {
            return glib::Propagation::Stop;
        }
        let handled = pressed
            .upgrade()
            .is_some_and(|view| key_pressed(&view, keyval, state, ctrl.im_context().as_ref()));
        glib::Propagation::from(handled)
    });
    let released = view.downgrade();
    keys.connect_key_released(move |_, keyval, _, state| {
        if let Some(view) = released.upgrade() {
            motion::sync_key_mods(&view, keyval, state, false);
        }
    });
    keys
}

fn key_pressed(
    view: &TermView,
    keyval: gdk::Key,
    state: gdk::ModifierType,
    im: Option<&gtk::IMContext>,
) -> bool {
    motion::sync_key_mods(view, keyval, state, true);
    if view.picture_open() {
        return true;
    }
    let mods = state & gtk::accelerator_get_default_mod_mask();
    if view.imp().bound.borrow().is_none()
        && mods == PASTE_SHORTCUT
        && keyval.to_lower() == gdk::Key::v
    {
        paste(view);
        return true;
    }
    let _ = im;
    match keys::bytes(keyval, mods, modes(view)) {
        Some(bytes) => {
            live::write(view, bytes);
            true
        }
        None => false,
    }
}

pub(super) fn modes(view: &TermView) -> Modes {
    live::with_terminal(view, |_, terminal, _| terminal.screen().modes()).unwrap_or_default()
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
    let entered_im = im.clone();
    let entered_view = view.downgrade();
    focus.connect_enter(move |focus| {
        entered_im.focus_in();
        if let Some(view) = entered_view.upgrade() {
            live::notify_focus(&view, true);
        }
        if let Some(widget) = focus.widget() {
            widget.queue_draw();
        }
    });
    let left_im = im.clone();
    let left_view = view.downgrade();
    focus.connect_leave(move |focus| {
        left_im.focus_out();
        if let Some(view) = left_view.upgrade() {
            motion::clear_mods(&view);
            hover::clear_hover(&view);
            live::notify_focus(&view, false);
        }
        if let Some(widget) = focus.widget() {
            widget.queue_draw();
        }
    });
    focus
}
