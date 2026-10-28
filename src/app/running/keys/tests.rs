use trame::input::Command;
use trame::term::{SelectionKind, Terminal};
use winit::keyboard::ModifiersState;

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
