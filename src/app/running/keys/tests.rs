use std::cell::Cell;

use trame::input::{Command, Keymap, Overrides};
use trame::term::{SelectionKind, Terminal};
use trame::ui::Platform;
use winit::keyboard::ModifiersState;

use super::Memo;
use super::copy::{after_copy, takes_key};

const CONTROL: ModifiersState = ModifiersState::CONTROL;

fn selected(text: &str) -> Terminal {
    let mut terminal = Terminal::new(2, 10, 0);
    terminal.feed(text.as_bytes());
    let screen = terminal.screen_mut();
    screen.start_selection(SelectionKind::Simple, 0, 0);
    screen.extend_selection(0, 9);
    terminal
}

#[test]
fn ctrl_c_interrupts_after_a_copy() {
    let mut terminal = selected("hello");
    assert!(takes_key(Command::Copy, CONTROL, &terminal));
    after_copy(&mut terminal, CONTROL);
    assert!(!takes_key(Command::Copy, CONTROL, &terminal));
}

#[test]
fn ctrl_c_interrupts_over_a_blank_selection() {
    assert!(!takes_key(Command::Copy, CONTROL, &selected("")));
}

#[test]
fn ctrl_shift_c_keeps_the_selection() {
    let mut terminal = selected("hello");
    after_copy(&mut terminal, CONTROL | ModifiersState::SHIFT);
    assert_eq!(terminal.screen().selected_text().as_deref(), Some("hello"));
}

#[test]
fn the_keymap_is_built_once_per_change() {
    let builds = Cell::new(0);
    let build = |overrides: &Overrides| {
        builds.set(builds.get() + 1);
        Keymap::new(Platform::Windows, overrides)
    };
    let defaults = Overrides::default();
    let changed = defaults.with_keys(0, Vec::new(), Platform::Windows);
    let mut keymap = Memo::default();
    for overrides in [&defaults, &defaults, &changed, &changed, &defaults] {
        keymap.get(overrides, build);
    }
    assert_eq!(builds.get(), 3);
    let expected = Keymap::new(Platform::Windows, &defaults);
    assert_eq!(keymap.get(&defaults, build), &expected);
    assert_eq!(builds.get(), 3);
}
