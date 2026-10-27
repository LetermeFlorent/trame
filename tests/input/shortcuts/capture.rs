use trame::input::{Combo, Pressed, ShortcutInput, pressed};
use trame::ui::Platform;
use winit::keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey};

use super::{ALT, CONTROL, SHIFT};
use crate::character;

fn capture(
    platform: Platform,
    modifiers: ModifiersState,
    key: (Key, Key),
    code: KeyCode,
) -> Pressed {
    let (key, base) = key;
    let input = ShortcutInput {
        key: &key,
        base: &base,
        physical: PhysicalKey::Code(code),
        modifiers,
    };
    pressed(input, platform)
}

fn combo(text: &str) -> Pressed {
    Pressed::Combo(Combo::parse(text).expect(text))
}

#[test]
fn records_the_pressed_combination() {
    let windows = Platform::Windows;
    let letter = capture(
        windows,
        CONTROL | SHIFT,
        (character("T"), character("t")),
        KeyCode::KeyT,
    );
    assert_eq!(letter, combo("Ctrl+Shift+T"));
    let digit = capture(
        windows,
        ALT,
        (character("&"), character("&")),
        KeyCode::Digit1,
    );
    assert_eq!(digit, combo("Alt+1"));
    let numpad = capture(
        windows,
        CONTROL,
        (character("0"), character("0")),
        KeyCode::Numpad0,
    );
    assert_eq!(numpad, combo("Ctrl+Num0"));
    let function = capture(
        windows,
        ModifiersState::empty(),
        named(NamedKey::F5),
        KeyCode::F5,
    );
    assert_eq!(function, combo("F5"));
    let page = capture(windows, SHIFT, named(NamedKey::PageUp), KeyCode::PageUp);
    assert_eq!(page, combo("Shift+PageUp"));
}

#[test]
fn waits_for_a_key_while_only_modifiers_are_held() {
    let control = capture(
        Platform::Windows,
        CONTROL,
        named(NamedKey::Control),
        KeyCode::ControlLeft,
    );
    assert_eq!(control, Pressed::Modifier);
    let shift = capture(
        Platform::Mac,
        SHIFT,
        named(NamedKey::Shift),
        KeyCode::ShiftLeft,
    );
    assert_eq!(shift, Pressed::Modifier);
}

#[test]
fn keeps_typing_keys_for_the_terminal() {
    let windows = Platform::Windows;
    let plain = capture(
        windows,
        ModifiersState::empty(),
        (character("a"), character("a")),
        KeyCode::KeyA,
    );
    assert_eq!(plain, Pressed::Typing);
    let shifted = capture(
        windows,
        SHIFT,
        (character("A"), character("a")),
        KeyCode::KeyA,
    );
    assert_eq!(shifted, Pressed::Typing);
    let enter = capture(
        windows,
        ModifiersState::empty(),
        named(NamedKey::Enter),
        KeyCode::Enter,
    );
    assert_eq!(enter, Pressed::Typing);
    let alt_graph = capture(
        windows,
        CONTROL | ALT,
        (character("@"), character("à")),
        KeyCode::Digit0,
    );
    assert_eq!(alt_graph, Pressed::Typing);
    let option = capture(
        Platform::Mac,
        ALT,
        (character("é"), character("e")),
        KeyCode::KeyE,
    );
    assert_eq!(option, Pressed::Typing);
}

#[test]
fn refuses_keys_it_cannot_name() {
    let media = capture(
        Platform::Windows,
        CONTROL,
        named(NamedKey::MediaPlayPause),
        KeyCode::MediaPlayPause,
    );
    assert_eq!(media, Pressed::Unsupported);
    let dead = (Key::Dead(Some('^')), Key::Dead(Some('^')));
    let accent = capture(Platform::Windows, CONTROL, dead, KeyCode::BracketLeft);
    assert_eq!(accent, Pressed::Unsupported);
}

fn named(key: NamedKey) -> (Key, Key) {
    (Key::Named(key), Key::Named(key))
}
