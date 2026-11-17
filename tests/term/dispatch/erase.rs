use trame::term::{Color, Flags, Terminal};

use crate::{feed, lines};

const SGR_RED: Color = Color::Indexed(1);

fn red_cells(term: &Terminal) -> String {
    let row = term.screen().grid().row(0);
    (0..term.screen().cols())
        .map(|col| {
            if row.cell(col).bg == SGR_RED {
                'r'
            } else {
                '.'
            }
        })
        .collect()
}

fn orphan_leader_then_x() -> Terminal {
    let mut term = feed(1, 4, "ab漢");
    term.screen_mut().resize(1, 3);
    term.feed(b"\x1b[1;1Ha");
    term.screen_mut().resize(1, 4);
    term.feed(b"\x1b[1;4Hx");
    let row = term.screen().grid().row(0);
    assert!(row.cell(2).flags.contains(Flags::WIDE));
    assert!(!row.cell(3).flags.contains(Flags::WIDE_SPACER));
    term
}

#[test]
fn a_leader_without_spacer_never_wipes_the_next_char() {
    let mut term = orphan_leader_then_x();
    term.feed(b"\x1b[1;3H\x1b[X");
    assert_eq!(lines(&term)[0], "ab x");
}

#[test]
fn a_leader_without_spacer_left_of_the_span_stays() {
    for control in ["X", "K", "P", "@"] {
        let mut term = orphan_leader_then_x();
        term.feed(format!("\x1b[1;4H\x1b[{control}").as_bytes());
        assert_eq!(lines(&term)[0], "ab漢", "{control}");
    }
}

#[test]
fn cut_wide_chars_clear_with_the_background() {
    let term = feed(1, 4, "漢\x1b[1;2H\x1b[41m\x1b[K");
    assert_eq!(
        (lines(&term)[0].as_str(), red_cells(&term).as_str()),
        ("", "rrrr")
    );
    let term = feed(1, 4, "x漢\x1b[1;2H\x1b[41m\x1b[X");
    assert_eq!(
        (lines(&term)[0].as_str(), red_cells(&term).as_str()),
        ("x", ".rr.")
    );
    let term = feed(1, 4, "漢x\x1b[1;2H\x1b[41m\x1b[P");
    assert_eq!(
        (lines(&term)[0].as_str(), red_cells(&term).as_str()),
        (" x", "r..r")
    );
}

#[test]
fn ich_over_the_room_left_clears_the_cut_char() {
    let term = feed(1, 4, "a漢b\x1b[1;3H\x1b[41m\x1b[3@");
    assert_eq!(
        (lines(&term)[0].as_str(), red_cells(&term).as_str()),
        ("a", ".rrr")
    );
}

#[test]
fn counts_past_the_line_end_clear_the_cut_char() {
    for control in ["X", "@", "P"] {
        let term = feed(1, 4, &format!("a漢\x1b[1;3H\x1b[65535{control}"));
        assert_eq!(lines(&term)[0], "a", "{control}");
    }
}

#[test]
fn ed_clears_the_wide_char_it_cuts() {
    let term = feed(2, 4, "ab\r\n漢\x1b[2;2H\x1b[J");
    assert_eq!(lines(&term), ["ab", ""]);
    let term = feed(2, 4, "ab\r\n漢x\x1b[2;1H\x1b[1J");
    assert_eq!(lines(&term), ["", "  x"]);
}

#[test]
fn ech_clears_the_wide_char_it_cuts() {
    let term = feed(1, 4, "漢\x1b[1;1H\x1b[X\x1b[1;1Hab");
    assert_eq!(lines(&term)[0], "ab");
}

#[test]
fn el_clears_the_wide_char_it_cuts() {
    let term = feed(1, 4, "漢\x1b[1;1H\x1b[1K\x1b[1;1Hab");
    assert_eq!(lines(&term)[0], "ab");
    let term = feed(1, 4, "漢\x1b[1;2H\x1b[K");
    assert_eq!(lines(&term)[0], "");
}

#[test]
fn ich_clears_the_wide_char_it_cuts() {
    let term = feed(1, 4, "漢\x1b[1;2H\x1b[@");
    assert_eq!(lines(&term)[0], "");
    let term = feed(1, 4, "ab漢\x1b[1;1H\x1b[@");
    assert_eq!(lines(&term)[0], " ab");
}

#[test]
fn dch_clears_the_wide_char_it_cuts() {
    let term = feed(1, 4, "x漢\x1b[1;2H\x1b[P\x1b[1;1Hab");
    assert_eq!(lines(&term)[0], "ab");
    let term = feed(1, 4, "漢x\x1b[1;2H\x1b[P");
    assert_eq!(lines(&term)[0], " x");
}
