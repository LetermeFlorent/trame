use trame::term::{Modes, Terminal};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use super::{CONTROL, NONE, SHIFT, bytes, send};
use crate::{character, modes};

fn console(key: NamedKey, modifiers: ModifiersState) -> Option<Vec<u8>> {
    send(
        Key::Named(key),
        None,
        modifiers,
        modes(&[Modes::WIN32_INPUT]),
    )
}

#[test]
fn the_console_asks_for_whole_key_records() {
    let mut terminal = Terminal::new(4, 20, 0);
    terminal.feed(b"\x1b[?9001h\x1b[?1004h");
    assert!(terminal.screen().modes().contains(Modes::WIN32_INPUT));
    terminal.feed(b"\x1b[?9001l");
    assert!(!terminal.screen().modes().contains(Modes::WIN32_INPUT));
}

#[test]
fn arrows_travel_as_one_press_and_release_record() {
    assert_eq!(
        console(NamedKey::ArrowUp, NONE),
        bytes("\x1b[38;72;0;1;256;1_\x1b[38;72;0;0;256;1_")
    );
    assert_eq!(
        console(NamedKey::ArrowDown, CONTROL),
        bytes("\x1b[40;80;0;1;264;1_\x1b[40;80;0;0;264;1_")
    );
}

#[test]
fn escape_and_function_keys_carry_their_codes() {
    assert_eq!(
        console(NamedKey::Escape, NONE),
        bytes("\x1b[27;1;27;1;0;1_\x1b[27;1;27;0;0;1_")
    );
    assert_eq!(
        console(NamedKey::F5, SHIFT),
        bytes("\x1b[116;63;0;1;16;1_\x1b[116;63;0;0;16;1_")
    );
}

#[test]
fn typed_text_and_enter_stay_plain() {
    let win32 = modes(&[Modes::WIN32_INPUT]);
    assert_eq!(send(character("a"), Some("a"), NONE, win32), bytes("a"));
    assert_eq!(console(NamedKey::Enter, NONE), bytes("\r"));
    assert_eq!(console(NamedKey::Backspace, NONE), bytes("\x7f"));
}

#[test]
fn shift_enter_is_a_win32_record() {
    assert_eq!(
        console(NamedKey::Enter, SHIFT),
        bytes("\x1b[13;28;13;1;16;1_\x1b[13;28;13;0;16;1_")
    );
    assert_eq!(
        console(NamedKey::Enter, CONTROL),
        bytes("\x1b[13;28;10;1;8;1_\x1b[13;28;10;0;8;1_")
    );
}

#[test]
fn modified_tab_backspace_and_space_are_records() {
    assert_eq!(
        console(NamedKey::Tab, SHIFT),
        bytes("\x1b[9;15;9;1;16;1_\x1b[9;15;9;0;16;1_")
    );
    assert_eq!(
        console(NamedKey::Tab, CONTROL),
        bytes("\x1b[9;15;0;1;8;1_\x1b[9;15;0;0;8;1_")
    );
    assert_eq!(
        console(NamedKey::Backspace, CONTROL),
        bytes("\x1b[8;14;127;1;8;1_\x1b[8;14;127;0;8;1_")
    );
    assert_eq!(
        console(NamedKey::Space, CONTROL),
        bytes("\x1b[32;57;32;1;8;1_\x1b[32;57;32;0;8;1_")
    );
}
