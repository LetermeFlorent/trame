use std::fmt::Write as _;

use trame::term::{Color, Flags, Terminal};

const DISTINCT_LINKS: usize = 256;

fn feed_screen(
    rows: usize,
    cols: usize,
    mut write_row: impl FnMut(&mut String, usize),
) -> Terminal {
    let mut out = String::new();
    for row in 0..rows {
        if row > 0 {
            out.push_str("\r\n");
        }
        write_row(&mut out, row);
    }
    let mut terminal = Terminal::new(rows, cols, 0);
    terminal.feed(out.as_bytes());
    terminal
}

pub fn styled(rows: usize, cols: usize) -> Terminal {
    let mut seq = 0usize;
    let terminal = feed_screen(rows, cols, |out, row| {
        let mut col = 0;
        while col < cols {
            let fg = ((row * 37 + col * 13) % 200 + 30) as u8;
            let bg = ((row * 17 + col * 29) % 180 + 20) as u8;
            let link = seq % DISTINCT_LINKS;
            seq += 1;
            let _ = write!(
                out,
                "\x1b]8;;https://example.com/{link}\x07\x1b[38;2;{fg};120;200;48;5;{bg}m"
            );
            if col + 1 < cols && col % 4 == 0 {
                out.push('\u{6F22}');
                col += 2;
            } else {
                out.push((b'a' + (col % 26) as u8) as char);
                col += 1;
            }
            out.push_str("\x1b]8;;\x07");
        }
    });
    for r in 0..rows {
        let line = terminal.screen().grid().row(r);
        assert_eq!(line.len(), cols);
        assert!(line.cells().iter().any(|c| c.flags.contains(Flags::WIDE)));
        assert!(
            line.cells()
                .iter()
                .all(|c| c.link != 0 && c.fg != Color::Foreground && c.bg != Color::Background)
        );
    }
    terminal
}

pub fn marked(rows: usize, cols: usize) -> Terminal {
    let terminal = feed_screen(rows, cols, |out, _| {
        for col in 0..cols {
            out.push((b'a' + (col % 26) as u8) as char);
            out.push_str("\u{0301}\u{0308}\u{0302}");
        }
    });
    for r in 0..rows {
        let row = terminal.screen().grid().row(r);
        assert_eq!(row.marked_width(), cols);
        assert!((0..cols).all(|c| !row.marks(c).is_empty()));
    }
    terminal
}
