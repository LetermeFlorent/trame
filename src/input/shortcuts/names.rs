use winit::keyboard::{KeyCode, NamedKey};

use super::combo::Trigger;

pub(super) const NAMED: [(NamedKey, &str); 27] = [
    (NamedKey::Tab, "Tab"),
    (NamedKey::Enter, "Enter"),
    (NamedKey::Space, "Space"),
    (NamedKey::Backspace, "Backspace"),
    (NamedKey::Delete, "Delete"),
    (NamedKey::Insert, "Insert"),
    (NamedKey::Home, "Home"),
    (NamedKey::End, "End"),
    (NamedKey::PageUp, "PageUp"),
    (NamedKey::PageDown, "PageDown"),
    (NamedKey::ArrowUp, "Up"),
    (NamedKey::ArrowDown, "Down"),
    (NamedKey::ArrowLeft, "Left"),
    (NamedKey::ArrowRight, "Right"),
    (NamedKey::Escape, "Escape"),
    (NamedKey::F1, "F1"),
    (NamedKey::F2, "F2"),
    (NamedKey::F3, "F3"),
    (NamedKey::F4, "F4"),
    (NamedKey::F5, "F5"),
    (NamedKey::F6, "F6"),
    (NamedKey::F7, "F7"),
    (NamedKey::F8, "F8"),
    (NamedKey::F9, "F9"),
    (NamedKey::F10, "F10"),
    (NamedKey::F11, "F11"),
    (NamedKey::F12, "F12"),
];

pub(super) const NUMPAD: [(KeyCode, &str); 15] = [
    (KeyCode::Numpad0, "Num0"),
    (KeyCode::Numpad1, "Num1"),
    (KeyCode::Numpad2, "Num2"),
    (KeyCode::Numpad3, "Num3"),
    (KeyCode::Numpad4, "Num4"),
    (KeyCode::Numpad5, "Num5"),
    (KeyCode::Numpad6, "Num6"),
    (KeyCode::Numpad7, "Num7"),
    (KeyCode::Numpad8, "Num8"),
    (KeyCode::Numpad9, "Num9"),
    (KeyCode::NumpadAdd, "Num+"),
    (KeyCode::NumpadSubtract, "Num-"),
    (KeyCode::NumpadMultiply, "Num*"),
    (KeyCode::NumpadDivide, "Num/"),
    (KeyCode::NumpadDecimal, "Num."),
];

pub(super) const DIGITS: [KeyCode; 10] = [
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

pub(super) const MODIFIER_KEYS: [NamedKey; 14] = [
    NamedKey::Hyper,
    NamedKey::Alt,
    NamedKey::AltGraph,
    NamedKey::CapsLock,
    NamedKey::Control,
    NamedKey::Fn,
    NamedKey::FnLock,
    NamedKey::NumLock,
    NamedKey::ScrollLock,
    NamedKey::Shift,
    NamedKey::Symbol,
    NamedKey::SymbolLock,
    NamedKey::Meta,
    NamedKey::Super,
];

pub(super) fn is_function_key(named: NamedKey) -> bool {
    NAMED[15..].iter().any(|(key, _)| *key == named)
}

impl Trigger {
    pub fn named_slot(self) -> Option<usize> {
        match self {
            Trigger::Named(named) => NAMED.iter().position(|(key, _)| *key == named),
            _ => None,
        }
    }
}
