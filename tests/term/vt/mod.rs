mod recorder;

use recorder::{Event, Recorder};

fn parse(bytes: &[u8]) -> Vec<Event> {
    let mut parser = trame::term::vt::Parser::new();
    let mut recorder = Recorder::default();
    parser.advance(&mut recorder, bytes);
    recorder.events
}

#[test]
fn prints_ascii_and_executes_controls() {
    let events = parse(b"a\r\n");
    assert_eq!(
        events,
        vec![
            Event::Print('a'),
            Event::Execute(b'\r'),
            Event::Execute(b'\n')
        ]
    );
}

#[test]
fn decodes_utf8_across_chunks() {
    let mut parser = trame::term::vt::Parser::new();
    let mut recorder = Recorder::default();
    let bytes = "é漢".as_bytes();
    parser.advance(&mut recorder, &bytes[..1]);
    parser.advance(&mut recorder, &bytes[1..4]);
    parser.advance(&mut recorder, &bytes[4..]);
    assert_eq!(recorder.events, vec![Event::Print('é'), Event::Print('漢')]);
}

#[test]
fn replaces_invalid_utf8_and_keeps_following_ascii() {
    let events = parse(b"\xC3A\xFF");
    assert_eq!(
        events,
        vec![
            Event::Print('\u{FFFD}'),
            Event::Print('A'),
            Event::Print('\u{FFFD}')
        ]
    );
}

#[test]
fn text_after_a_cut_sequence_keeps_its_accents() {
    assert_eq!(parse("é".as_bytes()), vec![Event::Print('é')]);
    assert_eq!(parse("[1é".as_bytes()), vec![Event::Print('é')]);
}

#[test]
fn dispatches_csi_with_params_and_private_marker() {
    assert_eq!(
        parse(b"\x1b[12;34H"),
        vec![Event::Csi(vec![vec![12], vec![34]], vec![], b'H')]
    );
    assert_eq!(
        parse(b"\x1b[?1049h"),
        vec![Event::Csi(vec![vec![1049]], vec![b'?'], b'h')]
    );
}

#[test]
fn keeps_empty_params_as_zero() {
    assert_eq!(parse(b"\x1b[m"), vec![Event::Csi(vec![], vec![], b'm')]);
    assert_eq!(
        parse(b"\x1b[;5H"),
        vec![Event::Csi(vec![vec![0], vec![5]], vec![], b'H')]
    );
}

#[test]
fn groups_colon_subparams() {
    assert_eq!(
        parse(b"\x1b[38:2::255:0:10;1m"),
        vec![Event::Csi(
            vec![vec![38, 2, 0, 255, 0, 10], vec![1]],
            vec![],
            b'm'
        )]
    );
}

#[test]
fn collects_intermediates() {
    assert_eq!(
        parse(b"\x1b[2 q"),
        vec![Event::Csi(vec![vec![2]], vec![b' '], b'q')]
    );
    assert_eq!(parse(b"\x1b(0"), vec![Event::Esc(vec![b'('], b'0')]);
}

#[test]
fn dispatches_osc_with_bell_and_string_terminator() {
    assert_eq!(
        parse(b"\x1b]0;hello;world\x07"),
        vec![Event::Osc(
            vec![b"0".to_vec(), b"hello".to_vec(), b"world".to_vec()],
            true
        )]
    );
    assert_eq!(
        parse(b"\x1b]2;titre\x1b\\x"),
        vec![
            Event::Osc(vec![b"2".to_vec(), "titre".as_bytes().to_vec()], false),
            Event::Esc(vec![], b'\\'),
            Event::Print('x'),
        ]
    );
}

#[test]
fn cancel_aborts_sequence() {
    assert_eq!(
        parse(b"\x1b[12\x18A"),
        vec![Event::Execute(0x18), Event::Print('A')]
    );
}

#[test]
fn ignores_sequences_with_too_many_params() {
    let mut bytes = b"\x1b[".to_vec();
    for _ in 0..40 {
        bytes.extend_from_slice(b"1;");
    }
    bytes.extend_from_slice(b"mZ");
    assert_eq!(parse(&bytes), vec![Event::Print('Z')]);
}

#[test]
fn skips_dcs_until_string_terminator() {
    assert_eq!(
        parse(b"\x1bP1$qm\x1b\\ok"),
        vec![
            Event::Esc(vec![], b'\\'),
            Event::Print('o'),
            Event::Print('k')
        ]
    );
}

#[test]
fn executes_controls_inside_csi() {
    assert_eq!(
        parse(b"\x1b[1\n2A"),
        vec![
            Event::Execute(b'\n'),
            Event::Csi(vec![vec![12]], vec![], b'A')
        ]
    );
}
