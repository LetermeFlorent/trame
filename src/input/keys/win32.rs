use winit::keyboard::{ModifiersState, NamedKey};

use super::win32_codes::{scan_code as scan, unicode, virtual_key as vk};

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
    let state = control_state(modifiers);
    let key = key_record(named, state)?;
    let state = state | if key.enhanced { ENHANCED_KEY } else { 0 };
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

fn key_record(named: NamedKey, state: u16) -> Option<KeyRecord> {
    navigation_record(named)
        .or_else(|| function_record(named))
        .or_else(|| editing_record(named, state))
}

fn editing_record(named: NamedKey, state: u16) -> Option<KeyRecord> {
    if state == 0 {
        return None;
    }
    let (virtual_key, scan_code, plain, with_control) = match named {
        NamedKey::Enter => (
            vk::RETURN,
            scan::ENTER,
            unicode::CARRIAGE_RETURN,
            unicode::LINE_FEED,
        ),
        NamedKey::Tab => (vk::TAB, scan::TAB, unicode::TAB, unicode::NONE),
        NamedKey::Backspace => (
            vk::BACK,
            scan::BACKSPACE,
            unicode::BACKSPACE,
            unicode::DELETE,
        ),
        NamedKey::Space => (vk::SPACE, scan::SPACE, unicode::SPACE, unicode::SPACE),
        _ => return None,
    };
    let character = if state & LEFT_CTRL_PRESSED == 0 {
        plain
    } else {
        with_control
    };
    Some(record(virtual_key, scan_code, character, false))
}

fn navigation_record(named: NamedKey) -> Option<KeyRecord> {
    let (virtual_key, scan_code) = match named {
        NamedKey::ArrowUp => (vk::UP, scan::UP),
        NamedKey::ArrowDown => (vk::DOWN, scan::DOWN),
        NamedKey::ArrowLeft => (vk::LEFT, scan::LEFT),
        NamedKey::ArrowRight => (vk::RIGHT, scan::RIGHT),
        NamedKey::Home => (vk::HOME, scan::HOME),
        NamedKey::End => (vk::END, scan::END),
        NamedKey::PageUp => (vk::PRIOR, scan::PAGE_UP),
        NamedKey::PageDown => (vk::NEXT, scan::PAGE_DOWN),
        NamedKey::Insert => (vk::INSERT, scan::INSERT),
        NamedKey::Delete => (vk::DELETE, scan::DELETE),
        NamedKey::Escape => {
            return Some(record(vk::ESCAPE, scan::ESCAPE, unicode::ESCAPE, false));
        }
        _ => return None,
    };
    Some(record(virtual_key, scan_code, unicode::NONE, true))
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
        NamedKey::F11 => return Some(record(vk::F11, scan::F11, unicode::NONE, false)),
        NamedKey::F12 => return Some(record(vk::F12, scan::F12, unicode::NONE, false)),
        _ => return None,
    };
    Some(record(
        vk::F1 + index,
        scan::F1 + index,
        unicode::NONE,
        false,
    ))
}
