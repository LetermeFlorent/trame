use super::parse;
use super::recorder::Event;

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
