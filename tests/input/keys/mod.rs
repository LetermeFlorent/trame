mod sequences;

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
    assert_eq!(
        send(character("a"), Some("a"), NONE, Modes::default()),
        bytes("a")
    );
    assert_eq!(
        send(character("é"), Some("é"), NONE, Modes::default()),
        bytes("é")
    );
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
    let new_line = send(
        Key::Named(NamedKey::Enter),
        None,
        NONE,
        modes(&[Modes::NEW_LINE]),
    );
    assert_eq!(new_line, bytes("\r\n"));
}

#[test]
fn sends_control_characters_from_the_base_letter() {
    assert_eq!(
        send(character("c"), Some("\u{3}"), CONTROL, Modes::default()),
        bytes("\x03")
    );
    assert_eq!(
        send(character("A"), None, CONTROL | SHIFT, Modes::default()),
        bytes("\x01")
    );
    assert_eq!(
        send(character("["), None, CONTROL, Modes::default()),
        bytes("\x1b")
    );
    assert_eq!(
        send(character("é"), Some("é"), CONTROL, Modes::default()),
        bytes("é")
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn prefixes_alt_with_escape() {
    let alt = ModifiersState::ALT;
    assert_eq!(
        send(character("x"), Some("x"), alt, Modes::default()),
        bytes("\x1bx")
    );
    assert_eq!(named(NamedKey::Backspace, alt), bytes("\x1b\x7f"));
}

#[cfg(windows)]
#[test]
fn types_alt_graph_characters_as_text() {
    let alt_graph = CONTROL | ModifiersState::ALT;
    assert_eq!(
        send(character("@"), Some("@"), alt_graph, Modes::default()),
        bytes("@")
    );
    assert_eq!(
        send(character("€"), Some("€"), alt_graph, Modes::default()),
        bytes("€")
    );
}
