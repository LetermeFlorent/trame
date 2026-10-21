use trame::input::paste_bytes;
use trame::term::Modes;

use crate::modes;

fn paste(text: &str, modes: Modes) -> String {
    String::from_utf8(paste_bytes(text, modes)).unwrap()
}

#[test]
fn turns_line_feeds_into_carriage_returns() {
    assert_eq!(paste("a\nb", Modes::default()), "a\rb");
    assert_eq!(paste("a\r\nb\r\n", Modes::default()), "a\rb\r");
    assert_eq!(paste("", Modes::default()), "");
}

#[test]
fn wraps_the_text_when_the_program_asks_for_it() {
    let bracketed = modes(&[Modes::BRACKETED_PASTE]);
    assert_eq!(paste("ls\n", bracketed), "\x1b[200~ls\r\x1b[201~");
}

#[test]
fn strips_escapes_so_a_paste_cannot_close_its_own_wrapper() {
    let bracketed = modes(&[Modes::BRACKETED_PASTE]);
    let hostile = "x\x1b[201~rm -rf ~\n";
    assert_eq!(paste(hostile, bracketed), "\x1b[200~x[201~rm -rf ~\r\x1b[201~");
}
