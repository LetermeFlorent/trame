use gtk::pango;
use trame::term::{Cell, Flags};
use trame::theme::Rgba;
use trame::ui::terminal::colors;

use super::Paint;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Style {
    pub color: Rgba,
    pub bold: bool,
    pub italic: bool,
    pub underline: pango::Underline,
    pub strike: bool,
}

fn underline_kind(flags: Flags, linked: bool) -> pango::Underline {
    if flags.contains(Flags::CURLY_UNDERLINE) {
        pango::Underline::Error
    } else if flags.contains(Flags::DOUBLE_UNDERLINE) {
        pango::Underline::Double
    } else if linked || flags.intersects(Flags::ANY_UNDERLINE) {
        pango::Underline::Single
    } else {
        pango::Underline::None
    }
}

pub(super) fn style(paint: &Paint, cell: &Cell, under_cursor: bool, linked: bool) -> Style {
    let (fg, _) = colors(paint.theme, cell);
    Style {
        color: if under_cursor {
            paint.theme.terminal
        } else {
            fg
        },
        bold: cell.flags.contains(Flags::BOLD),
        italic: cell.flags.contains(Flags::ITALIC),
        underline: underline_kind(cell.flags, linked),
        strike: cell.flags.contains(Flags::STRIKE),
    }
}

pub(super) fn font(paint: &Paint, style: Style) -> pango::FontDescription {
    let mut desc = paint.metrics.font.clone();
    if style.bold {
        desc.set_weight(pango::Weight::Bold);
    }
    if style.italic {
        desc.set_style(pango::Style::Italic);
    }
    desc
}

pub(super) fn attributes(style: Style) -> pango::AttrList {
    let list = pango::AttrList::new();
    if style.underline != pango::Underline::None {
        list.insert(pango::AttrInt::new_underline(style.underline));
    }
    if style.strike {
        list.insert(pango::AttrInt::new_strikethrough(true));
    }
    list
}
