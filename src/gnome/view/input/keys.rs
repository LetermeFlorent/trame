use gtk::gdk;
use trame::input::{KeyInput, encode};
use trame::term::Modes;
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

pub fn bytes(keyval: gdk::Key, state: gdk::ModifierType, modes: Modes) -> Option<Vec<u8>> {
    let key = key(keyval)?;
    let base = self::key(keyval.to_lower()).unwrap_or_else(|| key.clone());
    let text = keyval
        .to_unicode()
        .filter(|ch| !ch.is_control())
        .map(String::from);
    let input = KeyInput {
        key: &key,
        base: &base,
        text: text.as_deref(),
        modifiers: modifiers(state),
    };
    encode(input, modes)
}

fn key(keyval: gdk::Key) -> Option<Key> {
    if let Some(named) = named(keyval) {
        return Some(Key::Named(named));
    }
    let ch = keyval.to_unicode().filter(|ch| !ch.is_control())?;
    Some(Key::Character(SmolStr::new(ch.to_string())))
}

fn named(keyval: gdk::Key) -> Option<NamedKey> {
    editing(keyval)
        .or_else(|| moving(keyval))
        .or_else(|| function(keyval))
}

fn editing(keyval: gdk::Key) -> Option<NamedKey> {
    Some(match keyval {
        gdk::Key::Return | gdk::Key::KP_Enter | gdk::Key::ISO_Enter => NamedKey::Enter,
        gdk::Key::Tab | gdk::Key::ISO_Left_Tab | gdk::Key::KP_Tab => NamedKey::Tab,
        gdk::Key::BackSpace => NamedKey::Backspace,
        gdk::Key::Escape => NamedKey::Escape,
        gdk::Key::space | gdk::Key::KP_Space => NamedKey::Space,
        gdk::Key::Insert | gdk::Key::KP_Insert => NamedKey::Insert,
        gdk::Key::Delete | gdk::Key::KP_Delete => NamedKey::Delete,
        _ => return None,
    })
}

fn moving(keyval: gdk::Key) -> Option<NamedKey> {
    Some(match keyval {
        gdk::Key::Up | gdk::Key::KP_Up => NamedKey::ArrowUp,
        gdk::Key::Down | gdk::Key::KP_Down => NamedKey::ArrowDown,
        gdk::Key::Left | gdk::Key::KP_Left => NamedKey::ArrowLeft,
        gdk::Key::Right | gdk::Key::KP_Right => NamedKey::ArrowRight,
        gdk::Key::Home | gdk::Key::KP_Home => NamedKey::Home,
        gdk::Key::End | gdk::Key::KP_End => NamedKey::End,
        gdk::Key::Page_Up | gdk::Key::KP_Page_Up => NamedKey::PageUp,
        gdk::Key::Page_Down | gdk::Key::KP_Page_Down => NamedKey::PageDown,
        _ => return None,
    })
}

fn function(keyval: gdk::Key) -> Option<NamedKey> {
    const KEYS: [(gdk::Key, NamedKey); 12] = [
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
    ];
    KEYS.iter()
        .find(|(gdk_key, _)| *gdk_key == keyval)
        .map(|(_, named)| *named)
}

fn modifiers(state: gdk::ModifierType) -> ModifiersState {
    let pairs = [
        (gdk::ModifierType::SHIFT_MASK, ModifiersState::SHIFT),
        (gdk::ModifierType::CONTROL_MASK, ModifiersState::CONTROL),
        (gdk::ModifierType::ALT_MASK, ModifiersState::ALT),
        (gdk::ModifierType::SUPER_MASK, ModifiersState::SUPER),
    ];
    pairs
        .into_iter()
        .filter(|(mask, _)| state.contains(*mask))
        .fold(ModifiersState::empty(), |all, (_, one)| all | one)
}
