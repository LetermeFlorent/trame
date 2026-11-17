use super::parse;
use super::recorder::{Event, Recorder};

const MAX_APC_LEN: usize = 1 << 20;

fn apcs(events: &[Event]) -> Vec<&[u8]> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Apc(data) => Some(data.as_slice()),
            _ => None,
        })
        .collect()
}

#[test]
fn dispatches_apc_up_to_the_string_terminator() {
    assert_eq!(
        parse(b"\x1b_Ga=q;AAAA\x1b\\x"),
        vec![
            Event::Apc(b"Ga=q;AAAA".to_vec()),
            Event::Esc(vec![], b'\\'),
            Event::Print('x'),
        ]
    );
}

#[test]
fn reassembles_apc_cut_anywhere() {
    let bytes = b"a\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\b";
    let whole = parse(bytes);
    for size in 1..bytes.len() {
        let mut parser = trame::term::vt::Parser::new();
        let mut recorder = Recorder::default();
        for chunk in bytes.chunks(size) {
            parser.advance(&mut recorder, chunk);
        }
        assert_eq!(recorder.events, whole, "chunks of {size} bytes");
    }
}

#[test]
fn cancel_drops_the_apc_and_returns_to_ground() {
    assert_eq!(
        parse(b"\x1b_Ga=T;AA\x18A"),
        vec![Event::Execute(0x18), Event::Print('A')]
    );
    assert_eq!(
        parse(b"\x1b_Ga=T;AA\x1aA"),
        vec![Event::Execute(0x1A), Event::Print('A')]
    );
}

#[test]
fn drops_an_apc_longer_than_the_limit() {
    let mut bytes = b"\x1b_G".to_vec();
    bytes.resize(MAX_APC_LEN + 8, b'A');
    bytes.extend_from_slice(b"\x1b\\\x1b_Gok\x1b\\");
    assert_eq!(apcs(&parse(&bytes)), vec![&b"Gok"[..]]);
}

#[test]
fn keeps_eight_bit_bytes_and_skips_controls() {
    let events = parse("\x1b_Gé\r\n\u{80}\x1b\\".as_bytes());
    assert_eq!(apcs(&events), vec!["Gé\u{80}".as_bytes()]);
}

#[test]
fn other_control_strings_stay_ignored() {
    assert_eq!(
        parse(b"\x1bXsos\x1b\\\x1b^pm\x1b\\"),
        vec![Event::Esc(vec![], b'\\'), Event::Esc(vec![], b'\\')]
    );
}
