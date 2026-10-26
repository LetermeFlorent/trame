use winit::keyboard::{ModifiersState, NamedKey};

use super::keys::win32_key;
use crate::term::Modes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WheelAction {
    History(isize),
    Keys(usize, bool),
    Report,
    Ignore,
}

pub fn wheel_action(lines: isize, modes: Modes) -> WheelAction {
    if lines == 0 {
        return WheelAction::Ignore;
    }
    if modes.intersects(Modes::MOUSE_ANY) {
        return WheelAction::Report;
    }
    if !modes.contains(Modes::ALT_SCREEN) {
        return WheelAction::History(lines);
    }
    match modes.contains(Modes::ALTERNATE_SCROLL) {
        true => WheelAction::Keys(lines.unsigned_abs(), lines > 0),
        false => WheelAction::Ignore,
    }
}

pub fn arrow_keys(count: usize, up: bool, modes: Modes) -> Vec<u8> {
    let named = if up {
        NamedKey::ArrowUp
    } else {
        NamedKey::ArrowDown
    };
    if modes.contains(Modes::WIN32_INPUT)
        && let Some(key) = win32_key(named, ModifiersState::empty())
    {
        return key.repeat(count);
    }
    let letter = if up { 'A' } else { 'B' };
    let key = match modes.contains(Modes::APP_CURSOR) {
        true => format!("\x1bO{letter}"),
        false => format!("\x1b[{letter}"),
    };
    key.repeat(count).into_bytes()
}
