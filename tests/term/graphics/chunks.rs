use trame::term::Terminal;
use trame::term::graphics::{Chunks, Error, Format, MAX_ENCODED, parse, split};

fn chunk(apc: &[u8]) -> (trame::term::graphics::Command, &[u8]) {
    let (control, payload) = split(apc).unwrap();
    (parse(control), payload)
}

#[test]
fn reassembles_chunks_with_the_first_keys() {
    let (first, payload) = chunk(b"Gi=5,f=24,s=1,v=1,m=1;AQ");
    let mut chunks = Chunks::start(first);
    assert_eq!(chunks.push(true, payload), Ok(None));
    let (second, payload) = chunk(b"Gm=1;ID");
    assert_eq!(chunks.push(second.more, payload), Ok(None));
    let (last, payload) = chunk(b"Gm=0;==");
    let (command, raw) = chunks.push(last.more, payload).unwrap().unwrap();
    assert_eq!(raw, vec![1, 2, 3]);
    assert_eq!(command.id, 5);
    assert_eq!(command.format, Format::Rgb);
    assert!(!command.more);
}

#[test]
fn a_single_chunk_completes_at_once() {
    let (first, payload) = chunk(b"Gi=1;AQID");
    let mut chunks = Chunks::start(first);
    let (command, raw) = chunks.push(false, payload).unwrap().unwrap();
    assert_eq!((command.id, raw), (1, vec![1, 2, 3]));
}

#[test]
fn rejects_payloads_over_the_cap() {
    let mut chunks = Chunks::start(parse(b"i=1"));
    assert_eq!(chunks.push(true, &vec![b'A'; MAX_ENCODED - 4]), Ok(None));
    assert_eq!(chunks.push(true, b"AAAAAAAA"), Err(Error::TooBig));
}

#[test]
fn rejects_invalid_base64() {
    let mut chunks = Chunks::start(parse(b"i=1"));
    assert_eq!(chunks.push(false, b"A*B"), Err(Error::Inval));
}

#[test]
fn a_quiet_key_on_a_later_chunk_silences_the_reply() {
    let mut term = Terminal::new(4, 10, 0);
    term.feed(b"\x1b_Ga=t,i=6,f=24,s=1,v=1,m=1;AA\x1b\\\x1b_Gm=0,q=2;AA\x1b\\");
    assert!(term.screen_mut().take_responses().is_empty());
    assert!(term.screen().graphics().get(6).is_some());
}

#[test]
fn later_chunks_may_repeat_the_action() {
    let mut term = Terminal::new(4, 10, 0);
    term.feed(b"\x1b_Ga=T,i=2,f=24,s=1,v=1,m=1;AA\x1b\\\x1b_Ga=T,m=0;AA\x1b\\");
    assert_eq!(term.screen_mut().take_responses(), b"\x1b_Gi=2;OK\x1b\\");
    assert!(term.screen().graphics().get(2).is_some());
}
