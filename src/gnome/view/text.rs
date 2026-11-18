use gtk::prelude::*;
use gtk::{graphene, pango};
use trame::term::graphics::PLACEHOLDER;
use trame::term::grid::Row;
use trame::term::{Cell, Flags};
use trame::theme::Rgba;
use trame::ui::terminal::{CursorPlace, colors};

use super::paint::Paint;
use super::theme::rgba;

#[derive(Clone, Copy, PartialEq)]
struct Style {
    color: Rgba,
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
}

struct Run {
    col: usize,
    text: String,
    style: Style,
}

pub fn draw_row(
    paint: &Paint,
    row: usize,
    cells: &[Cell],
    line: Option<&Row>,
    block: Option<CursorPlace>,
) {
    let mut run: Option<Run> = None;
    for (col, cell) in cells.iter().enumerate() {
        if cell.flags.contains(Flags::WIDE_SPACER) {
            continue;
        }
        if cell.ch == PLACEHOLDER {
            flush(paint, row, run.take());
            continue;
        }
        let style = style(
            paint,
            cell,
            block.is_some_and(|place| place.covers(row, col)),
        );
        let mut text = String::from(if cell.ch == '\0' { ' ' } else { cell.ch });
        if let Some(line) = line {
            text.extend(line.marks(col).chars());
        }
        let alone = !text.is_ascii() || cell.flags.contains(Flags::WIDE);
        let joins = run.as_ref().is_some_and(|run| run.style == style) && !alone;
        if !joins {
            flush(paint, row, run.take());
        }
        if cell.flags.contains(Flags::HIDDEN) {
            continue;
        }
        if alone {
            flush(paint, row, Some(Run { col, text, style }));
            continue;
        }
        match run.as_mut() {
            Some(run) => run.text.push_str(&text),
            None => run = Some(Run { col, text, style }),
        }
    }
    flush(paint, row, run);
}

fn style(paint: &Paint, cell: &Cell, under_cursor: bool) -> Style {
    let (foreground, _) = colors(paint.theme, cell);
    Style {
        color: if under_cursor {
            paint.theme.terminal
        } else {
            foreground
        },
        bold: cell.flags.contains(Flags::BOLD),
        italic: cell.flags.contains(Flags::ITALIC),
        underline: cell.flags.intersects(Flags::ANY_UNDERLINE),
        strike: cell.flags.contains(Flags::STRIKE),
    }
}

fn flush(paint: &Paint, row: usize, run: Option<Run>) {
    let Some(run) = run else {
        return;
    };
    let decorated = run.style.underline || run.style.strike;
    if run.text.trim().is_empty() && !decorated {
        return;
    }
    let layout = paint.view.create_pango_layout(Some(&run.text));
    layout.set_font_description(Some(&font(paint, run.style)));
    layout.set_attributes(Some(&attributes(run.style)));
    let origin = paint.cell(row, run.col, 1);
    paint.snapshot.save();
    paint
        .snapshot
        .translate(&graphene::Point::new(origin.x(), origin.y()));
    paint
        .snapshot
        .append_layout(&layout, &rgba(run.style.color));
    paint.snapshot.restore();
}

fn font(paint: &Paint, style: Style) -> pango::FontDescription {
    let mut font = paint.metrics.font.clone();
    if style.bold {
        font.set_weight(pango::Weight::Bold);
    }
    if style.italic {
        font.set_style(pango::Style::Italic);
    }
    font
}

fn attributes(style: Style) -> pango::AttrList {
    let attributes = pango::AttrList::new();
    if style.underline {
        attributes.insert(pango::AttrInt::new_underline(pango::Underline::Single));
    }
    if style.strike {
        attributes.insert(pango::AttrInt::new_strikethrough(true));
    }
    attributes
}
