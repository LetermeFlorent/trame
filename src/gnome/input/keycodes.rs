use gtk::gdk;
use winit::keyboard::{KeyCode, NamedKey, NativeKeyCode, PhysicalKey};

const FIRST_EVDEV_DIGIT: u32 = 10;
const LAST_EVDEV_DIGIT: u32 = 19;

pub fn named_key(keyval: gdk::Key) -> Option<NamedKey> {
    Some(match keyval {
        gdk::Key::Return | gdk::Key::KP_Enter | gdk::Key::ISO_Enter => NamedKey::Enter,
        gdk::Key::Tab | gdk::Key::ISO_Left_Tab | gdk::Key::KP_Tab => NamedKey::Tab,
        gdk::Key::BackSpace => NamedKey::Backspace,
        gdk::Key::Escape => NamedKey::Escape,
        gdk::Key::space | gdk::Key::KP_Space => NamedKey::Space,
        gdk::Key::Insert | gdk::Key::KP_Insert => NamedKey::Insert,
        gdk::Key::Delete | gdk::Key::KP_Delete => NamedKey::Delete,
        gdk::Key::Up | gdk::Key::KP_Up => NamedKey::ArrowUp,
        gdk::Key::Down | gdk::Key::KP_Down => NamedKey::ArrowDown,
        gdk::Key::Left | gdk::Key::KP_Left => NamedKey::ArrowLeft,
        gdk::Key::Right | gdk::Key::KP_Right => NamedKey::ArrowRight,
        gdk::Key::Home | gdk::Key::KP_Home => NamedKey::Home,
        gdk::Key::End | gdk::Key::KP_End => NamedKey::End,
        gdk::Key::Page_Up | gdk::Key::KP_Page_Up => NamedKey::PageUp,
        gdk::Key::Page_Down | gdk::Key::KP_Page_Down => NamedKey::PageDown,
        _ => return extra_named(keyval),
    })
}

fn extra_named(keyval: gdk::Key) -> Option<NamedKey> {
    const EXTRA: [(gdk::Key, NamedKey); 22] = [
        (gdk::Key::F1, NamedKey::F1),
        (gdk::Key::F2, NamedKey::F2),
        (gdk::Key::F3, NamedKey::F3),
        (gdk::Key::F4, NamedKey::F4),
        (gdk::Key::F5, NamedKey::F5),
        (gdk::Key::F6, NamedKey::F6),
        (gdk::Key::F7, NamedKey::F7),
        (gdk::Key::F8, NamedKey::F8),
        (gdk::Key::F9, NamedKey::F9),
        (gdk::Key::F10, NamedKey::F10),
        (gdk::Key::F11, NamedKey::F11),
        (gdk::Key::F12, NamedKey::F12),
        (gdk::Key::Shift_L, NamedKey::Shift),
        (gdk::Key::Shift_R, NamedKey::Shift),
        (gdk::Key::Control_L, NamedKey::Control),
        (gdk::Key::Control_R, NamedKey::Control),
        (gdk::Key::Alt_L, NamedKey::Alt),
        (gdk::Key::Alt_R, NamedKey::Alt),
        (gdk::Key::ISO_Level3_Shift, NamedKey::AltGraph),
        (gdk::Key::Super_L, NamedKey::Super),
        (gdk::Key::Super_R, NamedKey::Super),
        (gdk::Key::Caps_Lock, NamedKey::CapsLock),
    ];
    EXTRA.iter().find(|(k, _)| *k == keyval).map(|(_, n)| *n)
}

pub fn physical_of(keyval: gdk::Key, keycode: u32) -> PhysicalKey {
    const DIGITS: [KeyCode; 10] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ];
    if (FIRST_EVDEV_DIGIT..=LAST_EVDEV_DIGIT).contains(&keycode) {
        return PhysicalKey::Code(DIGITS[(keycode - FIRST_EVDEV_DIGIT) as usize]);
    }
    const NUMPAD: [(gdk::Key, KeyCode); 15] = [
        (gdk::Key::KP_0, KeyCode::Numpad0),
        (gdk::Key::KP_1, KeyCode::Numpad1),
        (gdk::Key::KP_2, KeyCode::Numpad2),
        (gdk::Key::KP_3, KeyCode::Numpad3),
        (gdk::Key::KP_4, KeyCode::Numpad4),
        (gdk::Key::KP_5, KeyCode::Numpad5),
        (gdk::Key::KP_6, KeyCode::Numpad6),
        (gdk::Key::KP_7, KeyCode::Numpad7),
        (gdk::Key::KP_8, KeyCode::Numpad8),
        (gdk::Key::KP_9, KeyCode::Numpad9),
        (gdk::Key::KP_Add, KeyCode::NumpadAdd),
        (gdk::Key::KP_Subtract, KeyCode::NumpadSubtract),
        (gdk::Key::KP_Multiply, KeyCode::NumpadMultiply),
        (gdk::Key::KP_Divide, KeyCode::NumpadDivide),
        (gdk::Key::KP_Decimal, KeyCode::NumpadDecimal),
    ];
    NUMPAD.iter().find(|(k, _)| *k == keyval).map_or(
        PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
        |(_, c)| PhysicalKey::Code(*c),
    )
}
