use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use trame::input::paste_bytes;
use trame::term::Modes;

use super::{TermView, keys, live};

const PASTE_MODS: gdk::ModifierType =
    gdk::ModifierType::CONTROL_MASK.union(gdk::ModifierType::SHIFT_MASK);

pub fn attach(view: &TermView) {
    let im = gtk::IMMulticontext::new();
    im.set_client_widget(Some(view));
    let typed = view.downgrade();
    im.connect_commit(move |_, text| {
        if let Some(view) = typed.upgrade() {
            live::write(&view, text.as_bytes().to_vec());
        }
    });
    let keys = gtk::EventControllerKey::new();
    keys.set_im_context(Some(&im));
    let pressed = view.downgrade();
    keys.connect_key_pressed(move |_, keyval, _, state| {
        let handled = pressed
            .upgrade()
            .is_some_and(|view| key_pressed(&view, keyval, state));
        glib::Propagation::from(handled)
    });
    view.add_controller(keys);
    view.add_controller(focus_watch(&im));
    view.add_controller(click_focus());
}

fn key_pressed(view: &TermView, keyval: gdk::Key, state: gdk::ModifierType) -> bool {
    let state = state & gtk::accelerator_get_default_mod_mask();
    if state == PASTE_MODS && keyval.to_lower() == gdk::Key::v {
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

fn modes(view: &TermView) -> Modes {
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

fn focus_watch(im: &gtk::IMMulticontext) -> gtk::EventControllerFocus {
    let focus = gtk::EventControllerFocus::new();
    let entered = im.clone();
    focus.connect_enter(move |focus| {
        entered.focus_in();
        if let Some(widget) = focus.widget() {
            widget.queue_draw();
        }
    });
    let left = im.clone();
    focus.connect_leave(move |focus| {
        left.focus_out();
        if let Some(widget) = focus.widget() {
            widget.queue_draw();
        }
    });
    focus
}

fn click_focus() -> gtk::GestureClick {
    let click = gtk::GestureClick::new();
    click.connect_pressed(|click, _, _, _| {
        if let Some(widget) = click.widget() {
            widget.grab_focus();
        }
    });
    click
}
