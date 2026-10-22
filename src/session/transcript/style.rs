use std::io::Write;

use crate::term::{Cell, Color, Flags};

const STYLE_FLAGS: [(Flags, &str); 9] = [
    (Flags::BOLD, ";1"),
    (Flags::DIM, ";2"),
    (Flags::ITALIC, ";3"),
    (Flags::UNDERLINE, ";4"),
    (Flags::DOUBLE_UNDERLINE, ";4:2"),
    (Flags::CURLY_UNDERLINE, ";4:3"),
    (Flags::INVERSE, ";7"),
    (Flags::HIDDEN, ";8"),
    (Flags::STRIKE, ";9"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Style {
    fg: Color,
    bg: Color,
    flags: Flags,
}

impl Style {
    pub(super) const PLAIN: Self = Self {
        fg: Color::Foreground,
        bg: Color::Background,
        flags: Flags::empty(),
    };

    pub(super) fn of(cell: &Cell) -> Self {
        let mut flags = cell.flags;
        flags.remove(Flags::WIDTH_MARKS);
        Self {
            fg: cell.fg,
            bg: cell.bg,
            flags,
        }
    }

    pub(super) fn write(self, out: &mut Vec<u8>) {
        out.extend_from_slice(b"\x1b[0");
        for (flag, code) in STYLE_FLAGS {
            if self.flags.contains(flag) {
                out.extend_from_slice(code.as_bytes());
            }
        }
        write_color(out, self.fg, 38);
        write_color(out, self.bg, 48);
        out.push(b'm');
    }
}

fn write_color(out: &mut Vec<u8>, color: Color, code: u8) {
    let _ = match color {
        Color::Indexed(index) => write!(out, ";{code};5;{index}"),
        Color::Rgb(r, g, b) => write!(out, ";{code};2;{r};{g};{b}"),
        Color::Foreground | Color::Background => Ok(()),
    };
}
