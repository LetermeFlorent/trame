use trame::desk::{Closing, Confirm, Effect};
use trame::locale::COUNT_SLOT;

use crate::tabs::opened;

const LINES: &str = "one\ntwo\nthree\nfour\nfive\u{7}";

#[test]
fn a_long_paste_asks_first() {
    let (mut desk, _) = opened();
    desk.preferences.confirm_paste = true;
    let effects = desk.paste(LINES.to_owned());
    assert_eq!(effects.as_slice(), [Effect::Changed]);
    let Some(Closing::Paste(pending)) = &desk.closing else {
        panic!("a pending paste");
    };
    let count = desk.strings.lines_in_all.replace(COUNT_SLOT, "5");
    assert!(pending.body.contains(&count));
    let question = desk.question().expect("a question");
    assert_eq!(question.confirm, Confirm::Paste);
}

#[test]
fn a_confirmed_paste_types_into_the_tab() {
    let (mut desk, id) = opened();
    desk.preferences.confirm_paste = true;
    let _ = desk.paste(LINES.to_owned());
    let effects = desk.settle_closing(true);
    assert_eq!(effects.as_slice(), [Effect::Typed(id), Effect::Changed]);
}

#[test]
fn a_paste_without_confirmation_types_at_once() {
    let (mut desk, id) = opened();
    desk.preferences.confirm_paste = false;
    let effects = desk.paste(LINES.to_owned());
    assert_eq!(effects.as_slice(), [Effect::Typed(id)]);
    assert!(desk.closing.is_none());
}
