use winit::keyboard::{Key, ModifiersState, NamedKey};

use super::keys::{KeyInput, encode};
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
    let key = Key::Named(if up {
        NamedKey::ArrowUp
    } else {
        NamedKey::ArrowDown
    });
    let input = KeyInput {
        key: &key,
        base: &key,
        text: None,
        modifiers: ModifiersState::empty(),
    };
    encode(input, modes).unwrap_or_default().repeat(count)
}
