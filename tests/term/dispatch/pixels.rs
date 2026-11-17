use trame::term::Terminal;

fn responses(term: &mut Terminal, input: &str) -> String {
    term.feed(input.as_bytes());
    String::from_utf8(term.screen_mut().take_responses()).unwrap()
}

#[test]
fn pixel_sizes_are_not_reported_while_unknown() {
    let mut term = Terminal::new(5, 10, 0);
    assert_eq!(responses(&mut term, "\x1b[14t\x1b[16t"), "");
}

#[test]
fn reports_the_text_area_in_pixels() {
    let mut term = Terminal::new(5, 10, 0);
    term.screen_mut().set_cell_pixels(9, 18);
    assert_eq!(responses(&mut term, "\x1b[14t"), "\x1b[4;90;90t");
}

#[test]
fn reports_the_cell_size_in_pixels() {
    let mut term = Terminal::new(5, 10, 0);
    term.screen_mut().set_cell_pixels(9, 18);
    assert_eq!(responses(&mut term, "\x1b[16t"), "\x1b[6;18;9t");
}

#[test]
fn the_cell_size_survives_a_full_reset() {
    let mut term = Terminal::new(5, 10, 0);
    term.screen_mut().set_cell_pixels(9, 18);
    assert_eq!(responses(&mut term, "\x1bc\x1b[16t"), "\x1b[6;18;9t");
}

#[test]
fn still_reports_the_size_in_cells() {
    let mut term = Terminal::new(5, 10, 0);
    assert_eq!(responses(&mut term, "\x1b[18t"), "\x1b[8;5;10t");
}

#[test]
fn a_replayed_transcript_leaves_no_reply_behind() {
    let mut term = Terminal::new(5, 10, 0);
    term.replay(b"\x1b[c\x1b_Gi=31,s=1,v=1,a=q,f=24;AAAA\x1b\\");
    assert!(term.screen_mut().take_responses().is_empty());
}
