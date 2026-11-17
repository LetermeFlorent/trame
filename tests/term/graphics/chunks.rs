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
