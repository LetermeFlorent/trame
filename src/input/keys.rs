use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::term::Modes;

const ESC: u8 = 0x1b;

#[derive(Clone, Copy, Debug)]
pub struct KeyInput<'a> {
    pub key: &'a Key,
    pub base: &'a Key,
    pub text: Option<&'a str>,
    pub modifiers: ModifiersState,
}

pub fn encode(input: KeyInput, modes: Modes) -> Option<Vec<u8>> {
    let modifiers = input.modifiers;
    match input.key {
        Key::Named(named) => named_key(*named, input, modes),
        Key::Character(_) if is_alt_graph(input) => text(input),
        Key::Character(_) if modifiers.control_key() => control_key(input),
        Key::Character(_) if modifiers.alt_key() && meta_sends_escape() => {
            text(input).map(|bytes| escaped(&bytes))
        }
        _ => text(input),
    }
}

fn named_key(named: NamedKey, input: KeyInput, modes: Modes) -> Option<Vec<u8>> {
    let modifiers = input.modifiers;
    let alt = modifiers.alt_key() && meta_sends_escape();
    let simple = |bytes: &[u8]| Some(if alt { escaped(bytes) } else { bytes.to_vec() });
    match named {
        NamedKey::Enter if modes.contains(Modes::NEW_LINE) => simple(b"\r\n"),
        NamedKey::Enter => simple(b"\r"),
        NamedKey::Tab if modifiers.shift_key() => Some(b"\x1b[Z".to_vec()),
        NamedKey::Tab => simple(b"\t"),
        NamedKey::Backspace if modifiers.control_key() => simple(b"\x08"),
        NamedKey::Backspace => simple(b"\x7f"),
        NamedKey::Escape => simple(b"\x1b"),
        NamedKey::Space if modifiers.control_key() => simple(b"\0"),
        NamedKey::Space => simple(b" "),
        _ => cursor_or_function(named, modifiers, modes).or_else(|| text(input)),
    }
}

fn cursor_or_function(named: NamedKey, modifiers: ModifiersState, modes: Modes) -> Option<Vec<u8>> {
    let parameter = modifier_parameter(modifiers);
    if let Some(letter) = cursor_letter(named) {
        return Some(match parameter {
            Some(value) => format!("\x1b[1;{value}{letter}").into_bytes(),
            None if modes.contains(Modes::APP_CURSOR) => format!("\x1bO{letter}").into_bytes(),
            None => format!("\x1b[{letter}").into_bytes(),
        });
    }
    if let Some(letter) = function_letter(named) {
        return Some(match parameter {
            Some(value) => format!("\x1b[1;{value}{letter}").into_bytes(),
            None => format!("\x1bO{letter}").into_bytes(),
        });
    }
    let number = tilde_number(named)?;
    Some(match parameter {
        Some(value) => format!("\x1b[{number};{value}~").into_bytes(),
        None => format!("\x1b[{number}~").into_bytes(),
    })
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
    let value = 1
        + u8::from(modifiers.shift_key())
        + 2 * u8::from(modifiers.alt_key())
        + 4 * u8::from(modifiers.control_key());
    (value > 1).then_some(value)
}

fn control_key(input: KeyInput) -> Option<Vec<u8>> {
    let Key::Character(base) = input.base else {
        return text(input);
    };
    let mut chars = base.chars();
    let (Some(ch), None) = (chars.next(), chars.next()) else {
        return text(input);
    };
    let Some(byte) = control_byte(ch.to_ascii_lowercase()) else {
        return text(input);
    };
    let alt = input.modifiers.alt_key() && meta_sends_escape();
    Some(if alt { vec![ESC, byte] } else { vec![byte] })
}

fn control_byte(ch: char) -> Option<u8> {
    Some(match ch {
        'a'..='z' => ch as u8 - b'a' + 1,
        '@' | ' ' | '2' => 0,
        '[' | '3' => ESC,
        '\\' | '4' => 0x1c,
        ']' | '5' => 0x1d,
        '^' | '6' => 0x1e,
        '_' | '-' | '/' | '7' => 0x1f,
        '?' | '8' => 0x7f,
        _ => return None,
    })
}

fn is_alt_graph(input: KeyInput) -> bool {
    cfg!(windows)
        && input.modifiers.control_key()
        && input.modifiers.alt_key()
        && input
            .text
            .is_some_and(|text| text.chars().all(|ch| !ch.is_control()))
}

fn text(input: KeyInput) -> Option<Vec<u8>> {
    let text = input.text.or(match input.key {
        Key::Character(text) => Some(text.as_str()),
        _ => None,
    })?;
    (!text.is_empty()).then(|| text.as_bytes().to_vec())
}

fn escaped(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len() + 1);
    output.push(ESC);
    output.extend_from_slice(bytes);
    output
}

fn meta_sends_escape() -> bool {
    !cfg!(target_os = "macos")
}
