use trame::term::Modes;
use winit::keyboard::{Key, NamedKey};

use super::{CONTROL, NONE, SHIFT, bytes, named, send};
use crate::modes;

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
    let application = send(
        Key::Named(NamedKey::ArrowUp),
        None,
        NONE,
        modes(&[Modes::APP_CURSOR]),
    );
    assert_eq!(application, bytes("\x1bOA"));
    assert_eq!(named(NamedKey::ArrowLeft, SHIFT), bytes("\x1b[1;2D"));
    assert_eq!(named(NamedKey::ArrowRight, CONTROL), bytes("\x1b[1;5C"));
    assert_eq!(named(NamedKey::Home, NONE), bytes("\x1b[H"));
}

#[test]
fn keeps_modifiers_on_arrows_in_application_mode() {
    let input = send(
        Key::Named(NamedKey::ArrowDown),
        None,
        CONTROL,
        modes(&[Modes::APP_CURSOR]),
    );
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
