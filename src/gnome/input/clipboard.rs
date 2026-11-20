use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio};
use trame::files::{join_paths, save_capture};
use trame::log_error;

use super::keys::after_copy;
use crate::gnome::state::WindowState;

pub fn copy_active(state: &mut WindowState, display: Option<&gdk::Display>) {
    let Some(session) = state.desk.sessions.active(&state.desk.workspace) else {
        return;
    };
    let mut terminal = session.terminal();
    if let Some(text) = terminal.screen().selected_text() {
        write_text(&text, display);
    }
    after_copy(&mut terminal, state.modifiers);
}

pub fn sync_primary(text: &str, display: Option<&gdk::Display>) {
    if !text.is_empty()
        && let Some(display) = display
    {
        display.primary_clipboard().set_text(text);
    }
}

pub fn write_text(text: &str, display: Option<&gdk::Display>) {
    if let Some(display) = display {
        display.clipboard().set_text(text);
        return;
    }
    if let Err(err) = arboard::Clipboard::new().and_then(|mut c| c.set_text(text)) {
        log_error!("the clipboard could not be written: {err}");
    }
}

pub fn paste_into_terminal(
    state: &Rc<RefCell<WindowState>>,
    display: Option<&gdk::Display>,
    redraw: impl Fn() + 'static,
) {
    if let Some(text) = read_arboard_terminal() {
        let mut s = state.borrow_mut();
        let effects = s.desk.paste(text);
        s.apply(effects);
        drop(s);
        redraw();
        return;
    }
    let Some(display) = display else { return };
    let target = Rc::clone(state);
    display
        .clipboard()
        .read_text_async(None::<&gio::Cancellable>, move |res| {
            if let Ok(Some(text)) = res {
                let mut s = target.borrow_mut();
                let effects = s.desk.paste(text.into());
                s.apply(effects);
                drop(s);
                redraw();
            }
        });
}

pub fn paste_primary(
    state: &Rc<RefCell<WindowState>>,
    display: Option<&gdk::Display>,
    redraw: impl Fn() + 'static,
) {
    let Some(display) = display else { return };
    let target = Rc::clone(state);
    display
        .primary_clipboard()
        .read_text_async(None::<&gio::Cancellable>, move |res| {
            if let Ok(Some(text)) = res {
                let mut s = target.borrow_mut();
                let effects = s.desk.paste(text.into());
                s.apply(effects);
                drop(s);
                redraw();
            }
        });
}

pub fn read_plain_clipboard() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

fn read_arboard_terminal() -> Option<String> {
    let mut cb = arboard::Clipboard::new().ok()?;
    if let Ok(paths) = cb.get().file_list()
        && !paths.is_empty()
    {
        return Some(join_paths(&paths));
    }
    if let Ok(text) = cb.get_text() {
        return Some(text);
    }
    let img = cb.get_image().ok()?;
    match save_capture(img.width, img.height, &img.bytes) {
        Ok(path) => Some(path),
        Err(err) => {
            log_error!("the copied image could not be saved: {err}");
            None
        }
    }
}
