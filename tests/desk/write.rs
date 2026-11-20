use trame::desk::{Desk, Effect};
use trame::session::SessionId;
use trame::term::SelectionKind;

use crate::tabs::opened;

const SCROLLED: isize = 5;
const KEY: &[u8] = b"x";

fn scroll_back(desk: &Desk, id: SessionId) {
    let mut terminal = desk.sessions.get(id).expect("the shell").terminal();
    let history = b"\r\n".repeat(terminal.screen().rows() * 2);
    terminal.feed(&history);
    terminal.screen_mut().scroll_display(SCROLLED);
}

#[test]
fn writing_nothing_changes_nothing() {
    let (mut desk, _) = opened();
    assert!(desk.write_to_active(|_| Vec::new()).is_empty());
}

#[test]
fn writing_bytes_reports_the_typed_tab() {
    let (mut desk, id) = opened();
    let effects = desk.write_to_active(|_| b"true\r".to_vec());
    assert_eq!(effects.as_slice(), [Effect::Typed(id)]);
}

#[test]
fn a_key_typed_while_scrolled_back_redraws_at_the_bottom() {
    let (mut desk, id) = opened();
    scroll_back(&desk, id);
    let effects = desk.write_to_active(|_| KEY.to_vec());
    assert_eq!(effects.as_slice(), [Effect::Typed(id), Effect::Redraw]);
}

#[test]
fn a_key_typed_over_a_selection_redraws_without_it() {
    let (mut desk, id) = opened();
    scroll_back(&desk, id);
    {
        let mut terminal = desk.sessions.get(id).expect("the shell").terminal();
        let screen = terminal.screen_mut();
        screen.start_selection(SelectionKind::Line, 0, 0);
        screen.scroll_display(-SCROLLED);
    }
    let effects = desk.write_to_active(|_| KEY.to_vec());
    assert_eq!(effects.as_slice(), [Effect::Typed(id), Effect::Redraw]);
}
