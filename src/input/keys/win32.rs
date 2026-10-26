use winit::keyboard::{ModifiersState, NamedKey};

const LEFT_ALT_PRESSED: u16 = 0x02;
const LEFT_CTRL_PRESSED: u16 = 0x08;
const SHIFT_PRESSED: u16 = 0x10;
const ENHANCED_KEY: u16 = 0x100;

#[derive(Clone, Copy)]
struct KeyRecord {
    virtual_key: u16,
    scan_code: u16,
    unicode: u16,
    enhanced: bool,
}

const fn record(virtual_key: u16, scan_code: u16, unicode: u16, enhanced: bool) -> KeyRecord {
    KeyRecord {
        virtual_key,
        scan_code,
        unicode,
        enhanced,
    }
}

pub(in crate::input) fn win32_key(named: NamedKey, modifiers: ModifiersState) -> Option<Vec<u8>> {
    let key = key_record(named)?;
    let state = control_state(modifiers) | if key.enhanced { ENHANCED_KEY } else { 0 };
    let event = |down: u8| {
        format!(
            "\x1b[{};{};{};{down};{state};1_",
            key.virtual_key, key.scan_code, key.unicode
        )
    };
    Some(format!("{}{}", event(1), event(0)).into_bytes())
}

fn control_state(modifiers: ModifiersState) -> u16 {
    [
        (modifiers.shift_key(), SHIFT_PRESSED),
        (modifiers.control_key(), LEFT_CTRL_PRESSED),
        (modifiers.alt_key(), LEFT_ALT_PRESSED),
    ]
    .into_iter()
    .filter(|(pressed, _)| *pressed)
    .fold(0, |state, (_, flag)| state | flag)
}

fn key_record(named: NamedKey) -> Option<KeyRecord> {
    navigation_record(named).or_else(|| function_record(named))
}

fn navigation_record(named: NamedKey) -> Option<KeyRecord> {
    Some(match named {
        NamedKey::ArrowUp => record(0x26, 0x48, 0, true),
        NamedKey::ArrowDown => record(0x28, 0x50, 0, true),
        NamedKey::ArrowLeft => record(0x25, 0x4b, 0, true),
        NamedKey::ArrowRight => record(0x27, 0x4d, 0, true),
        NamedKey::Home => record(0x24, 0x47, 0, true),
        NamedKey::End => record(0x23, 0x4f, 0, true),
        NamedKey::PageUp => record(0x21, 0x49, 0, true),
        NamedKey::PageDown => record(0x22, 0x51, 0, true),
        NamedKey::Insert => record(0x2d, 0x52, 0, true),
        NamedKey::Delete => record(0x2e, 0x53, 0, true),
        NamedKey::Escape => record(0x1b, 0x01, 0x1b, false),
        _ => return None,
    })
}

fn function_record(named: NamedKey) -> Option<KeyRecord> {
    let index = match named {
        NamedKey::F1 => 0,
        NamedKey::F2 => 1,
        NamedKey::F3 => 2,
        NamedKey::F4 => 3,
        NamedKey::F5 => 4,
        NamedKey::F6 => 5,
        NamedKey::F7 => 6,
        NamedKey::F8 => 7,
        NamedKey::F9 => 8,
        NamedKey::F10 => 9,
        NamedKey::F11 => return Some(record(0x7a, 0x57, 0, false)),
        NamedKey::F12 => return Some(record(0x7b, 0x58, 0, false)),
        _ => return None,
    };
    Some(record(0x70 + index, 0x3b + index, 0, false))
}
