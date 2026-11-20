use trame::session::{MAX_TRANSCRIPT_BYTES, TranscriptCopy};
use trame::term::{Flags, SelectionKind, Terminal};

use super::{crc32, has_image, osc, png};
use crate::terminal;

fn padded_png(bytes: usize, tag: u8) -> Vec<u8> {
    let mut base = png(1, 1);
    let end = base.len() - 12;
    let iend = base.split_off(end);
    let mut chunk = Vec::with_capacity(bytes + 12);
    let mut body = b"tEXtkey\0".to_vec();
    body.resize(4 + bytes.max(5), b'A' + (tag % 26));
    body[4] = tag;
    chunk.extend_from_slice(&((body.len() - 4) as u32).to_be_bytes());
    chunk.extend_from_slice(&body);
    chunk.extend_from_slice(&crc32(&body).to_be_bytes());
    base.extend_from_slice(&chunk);
    base.extend_from_slice(&iend);
    base
}

#[test]
fn osc_1337_allows_up_to_16_mib_while_other_oscs_cap_at_1_mib() {
    let mut term = terminal(12, 40);
    term.feed(&osc("inline=1", &padded_png(1_500_000, 1)));
    assert!(has_image(&term));

    let mut dropped = terminal(12, 40);
    let huge = format!("\x1b]1337;File=inline=1:{}\x07", "A".repeat((16 << 20) + 1));
    dropped.feed(huge.as_bytes());
    assert!(!has_image(&dropped));

    let mut title = terminal(4, 20);
    title.feed(format!("\x1b]2;{}\x07", "x".repeat((1 << 20) + 1)).as_bytes());
    assert!(title.screen().title().is_none());
}

#[test]
fn preview_on_last_row_scrolls_into_history_with_same_id() {
    let mut term = terminal(10, 40);
    term.feed(b"\x1b[10;1H");
    term.feed(&osc("inline=1", &png(64, 32)));
    term.feed(b"\r\n1\r\n2\r\n3");
    let grid = term.screen().grid();
    assert!(grid.history_len() >= 8);
    let mut found = Vec::new();
    for line in grid.first_line()..grid.end_line() {
        let cell = grid.line(line).unwrap().cell(0);
        if cell.flags.contains(Flags::IMAGE) {
            found.push((cell.image_id(), cell.image_tile()));
        }
    }
    assert_eq!(found.len(), 8);
    assert!(found.iter().all(|(id, _)| *id == found[0].0));
}

#[test]
fn selection_and_transcript_replace_image_cells_with_spaces() {
    let mut term = terminal(10, 40);
    term.feed(&osc("inline=1", &png(8, 8)));
    term.feed(b"\x1b[1;17HX");
    let screen = term.screen_mut();
    screen.start_selection(SelectionKind::Simple, 0, 0);
    screen.extend_selection(0, 16);
    assert_eq!(screen.selected_text().as_deref(), Some("                X"));
    let transcript =
        String::from_utf8(TranscriptCopy::of(&term).encode(MAX_TRANSCRIPT_BYTES)).unwrap();
    assert!(!transcript.chars().any(|ch| u32::from(ch) >= 0x10_0000));
}

#[test]
fn forty_1_mib_images_stay_within_32_mib_and_erase_drops_unused_ids() {
    let mut term = Terminal::new(24, 80, 0);
    for idx in 0..40u8 {
        term.feed(&osc("inline=1", &padded_png(1 << 20, idx)));
    }
    let screen = term.screen();
    assert!(screen.image_bytes() <= 32 * 1024 * 1024);
    for row in 0..24 {
        if let Some(id) = screen.grid().row(row).cell(0).image_id() {
            assert!(screen.image(id).is_some());
        }
    }
    let mut erased = terminal(10, 40);
    erased.feed(&osc("inline=1", &png(8, 8)));
    assert_eq!(erased.screen().image_count(), 1);
    erased.feed(b"\x1b[2J");
    erased.compact();
    assert_eq!(erased.screen().image_count(), 0);
}

#[test]
fn full_reset_preserves_image_next_id() {
    let mut term = terminal(10, 40);
    term.feed(&osc("inline=1", &png(8, 8)));
    let first = term.screen().grid().row(0).cell(0).image_id().unwrap();
    term.feed(b"\x1bc");
    term.feed(&osc("inline=1", &png(8, 8)));
    let second = term.screen().grid().row(0).cell(0).image_id().unwrap();
    assert_ne!(first, second);
}
