use trame::term::Terminal;

use super::fixture::{all_placed, encoded_png};
use crate::cursor;

const SIZED: &str = "inline=1;width=4;height=2;preserveAspectRatio=0";

fn file(term: &mut Terminal, args: &str, payload: &str) {
    term.feed(format!("\x1b]1337;File={args}:{payload}\x07").as_bytes());
}

fn spans(term: &Terminal) -> Vec<(u16, u16, u16)> {
    term.screen()
        .grid()
        .anchors()
        .map(|anchor| (anchor.col, anchor.cols, anchor.rows))
        .collect()
}

#[test]
fn an_inline_file_anchors_at_the_cursor_and_moves_below_it() {
    let mut term = Terminal::new(10, 20, 0);
    term.feed(b"\x1b[2;3H");
    file(&mut term, SIZED, &encoded_png(3, 5));
    assert_eq!(spans(&term), vec![(2, 4, 2)]);
    assert_eq!(term.screen().grid().row(1).anchors().len(), 1);
    assert_eq!(cursor(&term), (3, 0));
    let image = all_placed(&term)[0].0;
    assert_eq!(term.screen().graphics().get(image).unwrap().width, 3);
}

#[test]
fn do_not_move_cursor_keeps_the_cursor() {
    let mut term = Terminal::new(10, 20, 0);
    term.feed(b"\x1b[2;3H");
    file(
        &mut term,
        &format!("{SIZED};doNotMoveCursor=1"),
        &encoded_png(1, 1),
    );
    assert_eq!(spans(&term), vec![(2, 4, 2)]);
    assert_eq!(cursor(&term), (1, 2));
}

#[test]
fn a_file_that_is_not_inline_is_ignored() {
    let mut term = Terminal::new(10, 20, 0);
    file(&mut term, "inline=0;width=4;height=2", &encoded_png(1, 1));
    assert!(spans(&term).is_empty());
    assert!(term.screen().graphics().is_empty());
    assert_eq!(cursor(&term), (0, 0));
}

#[test]
fn a_multipart_file_matches_a_single_file() {
    let mut single = Terminal::new(10, 20, 0);
    let mut parts = Terminal::new(10, 20, 0);
    let encoded = encoded_png(3, 5);
    file(&mut single, SIZED, &encoded);
    parts.feed(format!("\x1b]1337;MultipartFile={SIZED}\x07").as_bytes());
    let third = encoded.len() / 3;
    for part in [
        &encoded[..third],
        &encoded[third..2 * third],
        &encoded[2 * third..],
    ] {
        parts.feed(format!("\x1b]1337;FilePart={part}\x07").as_bytes());
    }
    parts.feed(b"\x1b]1337;FileEnd\x1b\\");
    assert_eq!(spans(&parts), spans(&single));
    assert_eq!(cursor(&parts), cursor(&single));
}

#[test]
fn an_image_file_may_exceed_the_usual_osc_limit() {
    let mut term = Terminal::new(10, 20, 0);
    let name = "A".repeat(3 << 19);
    file(
        &mut term,
        &format!("name={name};{SIZED}"),
        &encoded_png(1, 1),
    );
    assert_eq!(spans(&term), vec![(0, 4, 2)]);
    let title = "x".repeat(3 << 19);
    term.feed(format!("\x1b]2;{title}\x07").as_bytes());
    assert!(term.screen().title().is_none());
}

#[test]
fn the_auto_size_follows_the_cell_size_and_the_aspect_ratio() {
    let mut term = Terminal::new(10, 20, 0);
    term.screen_mut().set_cell_pixels(2, 4);
    file(&mut term, "inline=1", &encoded_png(6, 8));
    file(&mut term, "inline=1;width=6", &encoded_png(6, 8));
    file(
        &mut term,
        "inline=1;width=100%;height=8px;preserveAspectRatio=0",
        &encoded_png(1, 1),
    );
    assert_eq!(spans(&term), vec![(0, 3, 2), (0, 6, 4), (0, 20, 2)]);
}

#[test]
fn report_cell_size_answers_only_when_the_size_is_known() {
    let mut term = Terminal::new(10, 20, 0);
    term.feed(b"\x1b]1337;ReportCellSize\x07");
    assert!(term.screen_mut().take_responses().is_empty());
    term.screen_mut().set_cell_pixels(9, 18);
    term.feed(b"\x1b]1337;ReportCellSize\x07");
    assert_eq!(
        term.screen_mut().take_responses(),
        b"\x1b]1337;ReportCellSize=18;9\x07"
    );
}

#[test]
fn a_broken_file_is_ignored_and_text_still_prints() {
    let mut term = Terminal::new(10, 20, 0);
    file(&mut term, SIZED, "not*base64");
    file(&mut term, SIZED, "AAAAAAAA");
    term.feed(b"ok");
    assert!(term.screen().graphics().is_empty());
    assert_eq!(term.screen().grid().row(0).text().trim_end(), "ok");
}
