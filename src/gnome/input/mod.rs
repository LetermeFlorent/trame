pub mod clipboard;
pub mod drop_scroll;
mod keycodes;
pub mod keys;
pub mod scroll;
#[cfg(test)]
mod tests;
pub mod typing;

use gtk::gdk;
use trame::input::Command;

use crate::gnome::state::WindowState;
use keys::{DecodedKey, gdk_modifiers, takes_key};
use scroll::scroll_history;
use typing::editing_key;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureOutcome {
    Pass,
    Consumed,
    Copy,
    Paste,
    ClosePicture,
}

pub fn handle_capture_key(
    state: &mut WindowState,
    exiting: bool,
    decoded: &DecodedKey,
) -> CaptureOutcome {
    if let Some(modal) = modal_capture(state, decoded) {
        return modal;
    }
    state.modifiers = decoded.modifiers;
    let cmd = state
        .keymap()
        .command_for(decoded.shortcut_input())
        .filter(|c| claims(state, *c));
    match cmd {
        None => CaptureOutcome::Pass,
        Some(cmd) => run_shortcut(state, exiting, cmd),
    }
}

fn modal_capture(state: &mut WindowState, decoded: &DecodedKey) -> Option<CaptureOutcome> {
    if state.frame.blocked {
        return Some(CaptureOutcome::Consumed);
    }
    if state.desk.closing.is_some() {
        if decoded.is_enter() || decoded.is_escape() {
            let effects = state.desk.settle_closing(decoded.is_enter());
            state.apply(effects);
        }
        return Some(CaptureOutcome::Consumed);
    }
    if state.picture_open {
        let out = if decoded.is_escape() {
            CaptureOutcome::ClosePicture
        } else {
            CaptureOutcome::Consumed
        };
        return Some(out);
    }
    if decoded.is_escape() && state.cancel_drag() {
        return Some(CaptureOutcome::Consumed);
    }
    if state.menu.is_some() {
        if decoded.is_escape() {
            state.menu = None;
            state.changed();
        }
        return Some(CaptureOutcome::Consumed);
    }
    if editing_key(state, decoded) || settings_key(state, decoded) {
        return Some(CaptureOutcome::Consumed);
    }
    None
}

fn run_shortcut(state: &mut WindowState, exiting: bool, cmd: Command) -> CaptureOutcome {
    match cmd {
        Command::Copy => CaptureOutcome::Copy,
        Command::Paste => CaptureOutcome::Paste,
        Command::ScrollPageUp => scroll_active(state, 1, false, false),
        Command::ScrollPageDown => scroll_active(state, -1, false, false),
        Command::ScrollTop => scroll_active(state, 0, true, false),
        Command::ScrollBottom => scroll_active(state, 0, false, true),
        other => {
            state.run_command(exiting, other);
            CaptureOutcome::Consumed
        }
    }
}

fn claims(state: &WindowState, command: Command) -> bool {
    state
        .desk
        .sessions
        .active(&state.desk.workspace)
        .is_none_or(|s| takes_key(command, state.modifiers, &s.terminal()))
}

fn scroll_active(state: &mut WindowState, pages: isize, top: bool, bottom: bool) -> CaptureOutcome {
    let Some(session) = state.desk.sessions.active(&state.desk.workspace) else {
        return CaptureOutcome::Consumed;
    };
    let id = session.id();
    let mut terminal = session.terminal();
    let screen = terminal.screen_mut();
    if top {
        screen.scroll_display(screen.grid().history_len() as isize);
        state.motion.stop_scroll(id);
    } else if bottom {
        screen.reset_display();
        state.motion.stop_scroll(id);
    } else {
        let page = screen.rows().saturating_sub(1).max(1) as isize;
        let smooth = state.desk.preferences.smooth_scroll;
        scroll_history(&mut state.motion, id, screen, pages * page, smooth);
    }
    drop(terminal);
    state.dirty = true;
    CaptureOutcome::Consumed
}

fn settings_key(state: &mut WindowState, decoded: &DecodedKey) -> bool {
    if state.settings.is_none() {
        return false;
    }
    state.modifiers = decoded.modifiers;
    if state.capturing_key() {
        state.capture_shortcut(decoded.shortcut_input(), decoded.is_escape());
    } else if decoded.is_escape() {
        state.toggle_settings();
    }
    true
}

pub fn update_modifiers(state: &mut WindowState, mask: gdk::ModifierType) {
    let next = gdk_modifiers(mask);
    if next != state.modifiers {
        state.modifiers = next;
        state.refresh_hover();
    }
}
