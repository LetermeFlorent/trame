use trame::term::{MAX_LINKS, Terminal};
use trame::ui::terminal::links::{link_at, view_links};

use crate::feed;

fn ids(term: &Terminal, row: usize) -> Vec<u16> {
    let line = term.screen().grid().row(row);
    (0..line.len()).map(|col| line.cell(col).link).collect()
}

#[test]
fn an_osc_8_link_keeps_its_address_on_the_cells_it_covers() {
    let input = "\x1b]8;id=7;https://example.com/a;b\x1b\\link\x1b]8;;\x1b\\ text";
    let term = feed(2, 20, input);
    let ids = ids(&term, 0);
    assert!(ids[..4].iter().all(|&id| id != 0 && id == ids[0]));
    assert!(ids[4..].iter().all(|&id| id == 0));
    assert_eq!(term.screen().link(ids[0]), Some("https://example.com/a;b"));
    assert_eq!(
        link_at(term.screen(), 0, 2).as_deref(),
        Some("https://example.com/a;b")
    );
    assert_eq!(link_at(term.screen(), 0, 6), None);
    let spans = view_links(term.screen(), 0..2);
    assert_eq!(spans.len(), 1);
    assert_eq!((spans[0].row, spans[0].start, spans[0].end), (0, 0, 4));
}

#[test]
fn a_style_reset_inside_the_link_keeps_it_open() {
    let term = feed(1, 10, "\x1b]8;;http://a.example\x07a\x1b[0mb\x1b]8;;\x07c");
    let ids = ids(&term, 0);
    assert!(ids[0] != 0 && ids[1] == ids[0]);
    assert_eq!(ids[2], 0);
}

#[test]
fn the_text_of_an_osc_8_link_opens_its_address_not_itself() {
    let term = feed(
        1,
        30,
        "\x1b]8;;https://example.com\x07src/main.rs\x1b]8;;\x07",
    );
    assert_eq!(
        link_at(term.screen(), 0, 3).as_deref(),
        Some("https://example.com")
    );
    assert_eq!(view_links(term.screen(), 0..1).len(), 1);
}

#[test]
fn an_osc_8_link_that_is_not_web_is_not_a_link() {
    for address in [
        "file:///C:/dir/file.txt",
        "ftp://example.com/x",
        "vscode://file/x",
    ] {
        let input = format!("\x1b]8;;{address}\x07C:\\dir\\file.txt\x1b]8;;\x07");
        let term = feed(1, 30, &input);
        assert!(view_links(term.screen(), 0..1).is_empty(), "{address}");
        assert_eq!(link_at(term.screen(), 0, 2), None, "{address}");
    }
    let term = feed(1, 30, "plain C:\\dir\\file.txt and /usr/bin/env");
    assert!(view_links(term.screen(), 0..1).is_empty());
}

#[test]
fn an_address_with_control_characters_or_too_long_is_dropped() {
    let long = format!("https://example.com/{}", "x".repeat(4096));
    for address in ["https://a.example/\u{200b}x", "https://a.example/ x", &long] {
        let term = feed(1, 10, &format!("\x1b]8;;{address}\x1b\\ab\x1b]8;;\x1b\\"));
        assert!(ids(&term, 0).iter().all(|&id| id == 0), "{address}");
    }
}

#[test]
fn old_addresses_make_room_once_the_table_is_full() {
    let mut term = Terminal::new(1, 10, 0);
    let last = MAX_LINKS + 10;
    for index in 0..=last {
        let input = format!("\r\x1b]8;;https://example.com/{index}\x07x\x1b]8;;\x07");
        term.feed(input.as_bytes());
    }
    let id = ids(&term, 0)[0];
    let wanted = format!("https://example.com/{last}");
    assert_eq!(term.screen().link(id), Some(wanted.as_str()));
}
