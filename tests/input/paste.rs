use trame::input::{PREVIEW_CHARS, asks_before_pasting, paste_bytes, paste_preview};
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
    assert_eq!(
        paste(hostile, bracketed),
        "\x1b[200~x[201~rm -rf ~\r\x1b[201~"
    );
}

#[test]
fn a_bracketed_paste_drops_control_characters() {
    let bracketed = modes(&[Modes::BRACKETED_PASTE]);
    assert_eq!(
        paste("ls\x03curl x|sh\x7f\u{9b}201~\n", bracketed),
        "\x1b[200~lscurl x|sh201~\r\x1b[201~"
    );
}

#[test]
fn a_plain_paste_drops_control_characters() {
    assert_eq!(
        paste("a\tb\x00c\x1bd\x08\r\n", Modes::default()),
        "a\tbcd\r"
    );
}

#[test]
fn a_paste_with_a_line_break_asks_first() {
    assert!(asks_before_pasting("ls\n", Modes::default()));
    assert!(asks_before_pasting("a\rb", Modes::default()));
    assert!(!asks_before_pasting("ls -la\t", Modes::default()));
}

#[test]
fn a_bracketed_paste_asks_only_about_control_characters() {
    let bracketed = modes(&[Modes::BRACKETED_PASTE]);
    assert!(!asks_before_pasting("a\nb\r\n", bracketed));
    assert!(asks_before_pasting("a\x03b", bracketed));
    assert!(asks_before_pasting("a\u{9b}b", bracketed));
}

#[test]
fn a_preview_shows_the_first_lines_and_counts_them_all() {
    let preview = paste_preview("one\ttwo\r\nthree\rfour\n\nfive\n\n");
    assert_eq!(preview.lines, ["one two", "three", "four"]);
    assert_eq!(preview.line_count, 5);
    assert!(!preview.hidden_characters);
}

#[test]
fn a_preview_cuts_long_lines_and_spots_hidden_characters() {
    let long = "x".repeat(PREVIEW_CHARS);
    let preview = paste_preview(&format!("{long}y\x1b[31m"));
    assert_eq!(preview.lines, [format!("{long}\u{2026}")]);
    assert_eq!(preview.line_count, 1);
    assert!(preview.hidden_characters);
}
