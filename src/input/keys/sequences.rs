use winit::keyboard::{ModifiersState, NamedKey};

use crate::term::Modes;

const NO_MODIFIER: u8 = 1;
const ALT_WEIGHT: u8 = 2;
const CONTROL_WEIGHT: u8 = 4;

pub(super) fn cursor_or_function(
    named: NamedKey,
    modifiers: ModifiersState,
    modes: Modes,
) -> Option<Vec<u8>> {
    let parameter = modifier_parameter(modifiers);
    if let Some(letter) = cursor_letter(named) {
        return Some(match parameter {
            Some(value) => modified_letter(letter, value),
            None if modes.contains(Modes::APP_CURSOR) => ss3_letter(letter),
            None => format!("\x1b[{letter}").into_bytes(),
        });
    }
    if let Some(letter) = function_letter(named) {
        return Some(match parameter {
            Some(value) => modified_letter(letter, value),
            None => ss3_letter(letter),
        });
    }
    let number = tilde_number(named)?;
    Some(match parameter {
        Some(value) => format!("\x1b[{number};{value}~").into_bytes(),
        None => format!("\x1b[{number}~").into_bytes(),
    })
}

fn modified_letter(letter: char, value: u8) -> Vec<u8> {
    format!("\x1b[1;{value}{letter}").into_bytes()
}

fn ss3_letter(letter: char) -> Vec<u8> {
    format!("\x1bO{letter}").into_bytes()
}

fn cursor_letter(named: NamedKey) -> Option<char> {
    Some(match named {
        NamedKey::ArrowUp => 'A',
        NamedKey::ArrowDown => 'B',
        NamedKey::ArrowRight => 'C',
        NamedKey::ArrowLeft => 'D',
        NamedKey::Home => 'H',
        NamedKey::End => 'F',
        _ => return None,
    })
}

fn function_letter(named: NamedKey) -> Option<char> {
    Some(match named {
        NamedKey::F1 => 'P',
        NamedKey::F2 => 'Q',
        NamedKey::F3 => 'R',
        NamedKey::F4 => 'S',
        _ => return None,
    })
}

fn tilde_number(named: NamedKey) -> Option<u8> {
    Some(match named {
        NamedKey::Insert => 2,
        NamedKey::Delete => 3,
        NamedKey::PageUp => 5,
        NamedKey::PageDown => 6,
        NamedKey::F5 => 15,
        NamedKey::F6 => 17,
        NamedKey::F7 => 18,
        NamedKey::F8 => 19,
        NamedKey::F9 => 20,
        NamedKey::F10 => 21,
        NamedKey::F11 => 23,
        NamedKey::F12 => 24,
        _ => return None,
    })
}

fn modifier_parameter(modifiers: ModifiersState) -> Option<u8> {
    let value = NO_MODIFIER
        + u8::from(modifiers.shift_key())
        + ALT_WEIGHT * u8::from(modifiers.alt_key())
        + CONTROL_WEIGHT * u8::from(modifiers.control_key());
    (value > NO_MODIFIER).then_some(value)
}
