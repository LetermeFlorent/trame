use std::sync::Mutex;

use trame::session::{MAX_TRANSCRIPT_BYTES, TranscriptCopy};
use trame::term::Terminal;

#[test]
fn encoding_a_transcript_does_not_hold_the_lock() {
    let mut terminal = Terminal::new(6, 20, 100);
    terminal.feed(b"kept");
    let terminal = Mutex::new(terminal);
    let copy = TranscriptCopy::take(&terminal);
    let mut held = terminal
        .try_lock()
        .expect("the copy keeps the terminal locked");
    held.feed(b" and more");
    assert_eq!(copy.encode(MAX_TRANSCRIPT_BYTES), b"kept\r\n");
}
