use trame::input::{Command, ShortcutInput, shortcut};
use trame::ui::Platform;
use trame::workspace::Edge;
use winit::keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey};

use crate::character;

const CONTROL: ModifiersState = ModifiersState::CONTROL;
const SHIFT: ModifiersState = ModifiersState::SHIFT;
const ALT: ModifiersState = ModifiersState::ALT;
const LOGO: ModifiersState = ModifiersState::SUPER;

fn press_with_base(
    platform: Platform,
    modifiers: ModifiersState,
    key: Key,
    base: Key,
    physical: PhysicalKey,
) -> Option<Command> {
    let input = ShortcutInput {
        key: &key,
        base: &base,
        physical,
        modifiers,
    };
    shortcut(input, platform)
}

fn press(
    platform: Platform,
    modifiers: ModifiersState,
    key: Key,
    code: KeyCode,
) -> Option<Command> {
    press_with_base(
        platform,
        modifiers,
        key.clone(),
        key,
        PhysicalKey::Code(code),
    )
}

fn named(platform: Platform, modifiers: ModifiersState, key: NamedKey) -> Option<Command> {
    press(platform, modifiers, Key::Named(key), KeyCode::F24)
}

#[test]
fn opens_and_closes_tabs_with_the_platform_modifier() {
    let windows = press_with_base(
        Platform::Windows,
        CONTROL | SHIFT,
        character("T"),
        character("t"),
        PhysicalKey::Code(KeyCode::KeyT),
    );
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
fn splits_right_and_down() {
    let right = press(
        Platform::Windows,
        CONTROL | SHIFT,
        character("d"),
        KeyCode::KeyD,
    );
    assert_eq!(right, Some(Command::SplitRight));
    let down = press(
        Platform::Windows,
        CONTROL | SHIFT | ALT,
        character("d"),
        KeyCode::KeyD,
    );
    assert_eq!(down, Some(Command::SplitDown));
    let mac_down = press(Platform::Mac, LOGO | SHIFT, character("d"), KeyCode::KeyD);
    assert_eq!(mac_down, Some(Command::SplitDown));
}

#[test]
fn selects_tabs_by_physical_digit_whatever_the_layout_prints() {
    let first = press(Platform::Windows, ALT, character("&"), KeyCode::Digit1);
    assert_eq!(first, Some(Command::SelectTab(0)));
    let third = press(Platform::Mac, LOGO, character("3"), KeyCode::Digit3);
    assert_eq!(third, Some(Command::SelectTab(2)));
    let last = press(Platform::Linux, ALT, character("9"), KeyCode::Digit9);
    assert_eq!(last, Some(Command::LastTab));
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

#[test]
fn changes_the_font_size() {
    let bigger = press(Platform::Windows, CONTROL, character("="), KeyCode::Equal);
    assert_eq!(bigger, Some(Command::FontBigger));
    let plus = press(
        Platform::Linux,
        CONTROL | SHIFT,
        character("+"),
        KeyCode::Equal,
    );
    assert_eq!(plus, Some(Command::FontBigger));
    let azerty_minus = press(Platform::Windows, CONTROL, character("-"), KeyCode::Digit6);
    assert_eq!(azerty_minus, Some(Command::FontSmaller));
    let numpad = press(
        Platform::Windows,
        CONTROL,
        character("+"),
        KeyCode::NumpadAdd,
    );
    assert_eq!(numpad, Some(Command::FontBigger));
    let azerty_reset = press(Platform::Windows, CONTROL, character("à"), KeyCode::Digit0);
    assert_eq!(azerty_reset, Some(Command::FontReset));
    let mac_reset = press(Platform::Mac, LOGO, character("0"), KeyCode::Digit0);
    assert_eq!(mac_reset, Some(Command::FontReset));
}

#[test]
fn reads_the_font_keys_when_the_position_is_unknown() {
    let unknown = PhysicalKey::Unidentified(winit::keyboard::NativeKeyCode::Unidentified);
    let bigger = press_with_base(
        Platform::Windows,
        CONTROL,
        character("+"),
        character("="),
        unknown,
    );
    assert_eq!(bigger, Some(Command::FontBigger));
}

#[test]
fn moves_focus_between_panes() {
    let left = named(Platform::Windows, CONTROL | ALT, NamedKey::ArrowLeft);
    assert_eq!(left, Some(Command::FocusPane(Edge::Left)));
    let right = named(Platform::Mac, LOGO | ALT, NamedKey::ArrowRight);
    assert_eq!(right, Some(Command::FocusPane(Edge::Right)));
    let down = named(Platform::Linux, CONTROL | ALT, NamedKey::ArrowDown);
    assert_eq!(down, Some(Command::FocusPane(Edge::Bottom)));
    assert_eq!(named(Platform::Windows, ALT, NamedKey::ArrowUp), None);
}

#[test]
fn cycles_tabs_and_instances() {
    let windows = Platform::Windows;
    assert_eq!(
        named(windows, CONTROL, NamedKey::Tab),
        Some(Command::NextTab)
    );
    assert_eq!(
        named(windows, CONTROL | SHIFT, NamedKey::Tab),
        Some(Command::PreviousTab)
    );
    assert_eq!(
        named(windows, CONTROL, NamedKey::PageDown),
        Some(Command::NextTab)
    );
    let previous = named(windows, CONTROL | SHIFT, NamedKey::PageUp);
    assert_eq!(previous, Some(Command::PreviousInstance));
}

#[test]
fn scrolls_and_pastes_from_the_keyboard() {
    let windows = Platform::Windows;
    assert_eq!(
        named(windows, SHIFT, NamedKey::PageUp),
        Some(Command::ScrollPageUp)
    );
    assert_eq!(
        named(windows, CONTROL | SHIFT, NamedKey::Home),
        Some(Command::ScrollTop)
    );
    assert_eq!(
        named(windows, CONTROL | SHIFT, NamedKey::End),
        Some(Command::ScrollBottom)
    );
    assert_eq!(
        named(windows, SHIFT, NamedKey::Insert),
        Some(Command::Paste)
    );
    assert_eq!(named(windows, SHIFT, NamedKey::Home), None);
    assert_eq!(
        named(windows, ModifiersState::empty(), NamedKey::PageUp),
        None
    );
}
