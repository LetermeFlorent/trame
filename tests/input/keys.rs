use trame::input::{KeyInput, encode};
use trame::term::Modes;
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::{character, modes};

const NONE: ModifiersState = ModifiersState::empty();
const CONTROL: ModifiersState = ModifiersState::CONTROL;
const SHIFT: ModifiersState = ModifiersState::SHIFT;

fn send(key: Key, text: Option<&str>, modifiers: ModifiersState, modes: Modes) -> Option<Vec<u8>> {
    let base = key.clone();
    let input = KeyInput {
        key: &key,
        base: &base,
        text,
        modifiers,
    };
    encode(input, modes)
}

fn named(key: NamedKey, modifiers: ModifiersState) -> Option<Vec<u8>> {
    send(Key::Named(key), None, modifiers, Modes::default())
}

fn bytes(value: &str) -> Option<Vec<u8>> {
    Some(value.as_bytes().to_vec())
}

#[test]
fn sends_typed_text() {
    assert_eq!(send(character("a"), Some("a"), NONE, Modes::default()), bytes("a"));
    assert_eq!(send(character("é"), Some("é"), NONE, Modes::default()), bytes("é"));
    assert_eq!(send(character(""), Some(""), NONE, Modes::default()), None);
}

#[test]
fn sends_nothing_for_a_lone_modifier() {
    assert_eq!(named(NamedKey::Shift, SHIFT), None);
    assert_eq!(named(NamedKey::Control, CONTROL), None);
}

#[test]
fn sends_enter_as_carriage_return_unless_new_line_mode() {
    assert_eq!(named(NamedKey::Enter, NONE), bytes("\r"));
    let new_line = send(Key::Named(NamedKey::Enter), None, NONE, modes(&[Modes::NEW_LINE]));
    assert_eq!(new_line, bytes("\r\n"));
}

#[test]
fn sends_editing_keys() {
    assert_eq!(named(NamedKey::Backspace, NONE), bytes("\x7f"));
    assert_eq!(named(NamedKey::Backspace, CONTROL), bytes("\x08"));
    assert_eq!(named(NamedKey::Tab, NONE), bytes("\t"));
    assert_eq!(named(NamedKey::Tab, SHIFT), bytes("\x1b[Z"));
    assert_eq!(named(NamedKey::Escape, NONE), bytes("\x1b"));
    assert_eq!(named(NamedKey::Space, CONTROL), bytes("\0"));
    assert_eq!(named(NamedKey::Delete, NONE), bytes("\x1b[3~"));
    assert_eq!(named(NamedKey::PageUp, NONE), bytes("\x1b[5~"));
}

#[test]
fn sends_arrows_in_normal_and_application_mode() {
    assert_eq!(named(NamedKey::ArrowUp, NONE), bytes("\x1b[A"));
    let application = send(Key::Named(NamedKey::ArrowUp), None, NONE, modes(&[Modes::APP_CURSOR]));
    assert_eq!(application, bytes("\x1bOA"));
    assert_eq!(named(NamedKey::ArrowLeft, SHIFT), bytes("\x1b[1;2D"));
    assert_eq!(named(NamedKey::ArrowRight, CONTROL), bytes("\x1b[1;5C"));
    assert_eq!(named(NamedKey::Home, NONE), bytes("\x1b[H"));
}

#[test]
fn keeps_modifiers_on_arrows_in_application_mode() {
    let input = send(Key::Named(NamedKey::ArrowDown), None, CONTROL, modes(&[Modes::APP_CURSOR]));
    assert_eq!(input, bytes("\x1b[1;5B"));
}

#[test]
fn sends_function_keys() {
    assert_eq!(named(NamedKey::F1, NONE), bytes("\x1bOP"));
    assert_eq!(named(NamedKey::F4, SHIFT), bytes("\x1b[1;2S"));
    assert_eq!(named(NamedKey::F5, NONE), bytes("\x1b[15~"));
    assert_eq!(named(NamedKey::F5, CONTROL), bytes("\x1b[15;5~"));
    assert_eq!(named(NamedKey::F12, NONE), bytes("\x1b[24~"));
}

#[test]
fn sends_control_characters_from_the_base_letter() {
    assert_eq!(send(character("c"), Some("\u{3}"), CONTROL, Modes::default()), bytes("\x03"));
    assert_eq!(send(character("A"), None, CONTROL | SHIFT, Modes::default()), bytes("\x01"));
    assert_eq!(send(character("["), None, CONTROL, Modes::default()), bytes("\x1b"));
    assert_eq!(send(character("é"), Some("é"), CONTROL, Modes::default()), bytes("é"));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn prefixes_alt_with_escape() {
    let alt = ModifiersState::ALT;
    assert_eq!(send(character("x"), Some("x"), alt, Modes::default()), bytes("\x1bx"));
    assert_eq!(named(NamedKey::Backspace, alt), bytes("\x1b\x7f"));
}

#[cfg(windows)]
#[test]
fn types_alt_graph_characters_as_text() {
    let alt_graph = CONTROL | ModifiersState::ALT;
    assert_eq!(send(character("@"), Some("@"), alt_graph, Modes::default()), bytes("@"));
    assert_eq!(send(character("€"), Some("€"), alt_graph, Modes::default()), bytes("€"));
}
