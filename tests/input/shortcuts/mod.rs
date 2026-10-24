mod navigation;

use trame::input::{Command, ShortcutInput, shortcut};
use trame::ui::Platform;
use winit::keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey};

use crate::character;

const CONTROL: ModifiersState = ModifiersState::CONTROL;
const SHIFT: ModifiersState = ModifiersState::SHIFT;
const ALT: ModifiersState = ModifiersState::ALT;
const LOGO: ModifiersState = ModifiersState::SUPER;

fn press(
    platform: Platform,
    modifiers: ModifiersState,
    key: Key,
    code: KeyCode,
) -> Option<Command> {
    let input = ShortcutInput {
        key: &key,
        base: &key,
        physical: PhysicalKey::Code(code),
        modifiers,
    };
    shortcut(input, platform)
}

fn named(platform: Platform, modifiers: ModifiersState, key: NamedKey) -> Option<Command> {
    press(platform, modifiers, Key::Named(key), KeyCode::F24)
}

#[test]
fn opens_and_closes_tabs_with_the_platform_modifier() {
    let shifted = ShortcutInput {
        key: &character("T"),
        base: &character("t"),
        physical: PhysicalKey::Code(KeyCode::KeyT),
        modifiers: CONTROL | SHIFT,
    };
    let windows = shortcut(shifted, Platform::Windows);
    assert_eq!(windows, Some(Command::NewTab));
    let linux = press(
        Platform::Linux,
        CONTROL | SHIFT,
        character("w"),
        KeyCode::KeyW,
    );
    assert_eq!(linux, Some(Command::CloseTab));
    let mac = press(Platform::Mac, LOGO, character("t"), KeyCode::KeyT);
    assert_eq!(mac, Some(Command::NewTab));
}

#[test]
fn leaves_plain_control_letters_to_the_shell() {
    assert_eq!(
        press(Platform::Windows, CONTROL, character("t"), KeyCode::KeyT),
        None
    );
    assert_eq!(
        press(Platform::Linux, CONTROL, character("c"), KeyCode::KeyC),
        None
    );
    assert_eq!(
        press(Platform::Mac, CONTROL, character("t"), KeyCode::KeyT),
        None
    );
    assert_eq!(
        press(
            Platform::Windows,
            CONTROL | SHIFT,
            character("q"),
            KeyCode::KeyQ
        ),
        None
    );
}

#[test]
fn copies_and_pastes_with_the_system_keys() {
    let windows = Platform::Windows;
    let copy = press(windows, CONTROL, character("c"), KeyCode::KeyC);
    assert_eq!(copy, Some(Command::Copy));
    let paste = press(windows, CONTROL, character("v"), KeyCode::KeyV);
    assert_eq!(paste, Some(Command::Paste));
    let shifted = press(windows, CONTROL | SHIFT, character("v"), KeyCode::KeyV);
    assert_eq!(shifted, Some(Command::Paste));
    let linux = press(Platform::Linux, CONTROL, character("v"), KeyCode::KeyV);
    assert_eq!(linux, None);
    let mac = press(Platform::Mac, LOGO, character("c"), KeyCode::KeyC);
    assert_eq!(mac, Some(Command::Copy));
}

#[test]
fn reads_letters_from_the_layout_not_the_key_position() {
    let azerty_w = press(
        Platform::Windows,
        CONTROL | SHIFT,
        character("w"),
        KeyCode::KeyZ,
    );
    assert_eq!(azerty_w, Some(Command::CloseTab));
    let azerty_q = press(
        Platform::Windows,
        CONTROL | SHIFT,
        character("a"),
        KeyCode::KeyQ,
    );
    assert_eq!(azerty_q, None);
}

#[test]
fn ignores_alt_graph_characters() {
    let tilde = press(
        Platform::Windows,
        CONTROL | ALT,
        character("~"),
        KeyCode::Digit2,
    );
    assert_eq!(tilde, None);
    let at = press(
        Platform::Windows,
        CONTROL | ALT,
        character("@"),
        KeyCode::Digit0,
    );
    assert_eq!(at, None);
    let brace = press(
        Platform::Windows,
        CONTROL | ALT,
        character("}"),
        KeyCode::Equal,
    );
    assert_eq!(brace, None);
}
