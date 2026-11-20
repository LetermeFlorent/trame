use gtk::prelude::*;
use gtk::{graphene, pango};
use trame::term::grid::Row;
use trame::term::{Cell, Flags};
use trame::theme::Rgba;
use trame::ui::terminal::links::LinkSpan;
use trame::ui::terminal::{CursorPlace, colors};

use super::Paint;
use super::strips::RowCells;
use crate::gnome::view::theme::rgba;

const NUL_CHAR: char = '\0';

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

pub fn draw_row(paint: &Paint, item: &RowCells, line: Option<&Row>, block: Option<CursorPlace>) {
    let row = item.row as usize;
    let links = row_links(paint.links, item.row);
    let mut run: Option<Run> = None;
    for (col, cell) in item.cells.iter().enumerate() {
        if cell.flags.contains(Flags::WIDE_SPACER) {
            continue;
        }
        if cell.flags.contains(Flags::IMAGE) || cell.flags.contains(Flags::HIDDEN) {
            flush(paint, row, run.take());
            continue;
        }
        let under = block.is_some_and(|place| place.covers(row, col));
        let linked = links.iter().any(|span| span.covers(col));
        let next_style = style(paint, cell, under, linked);
        let text = cell_text(cell, line, col);
        let alone = !text.is_ascii() || cell.flags.contains(Flags::WIDE);
        if !run.as_ref().is_some_and(|cur| cur.style == next_style) || alone {
            flush(paint, row, run.take());
        }
        if alone {
            let single = Run {
                col,
                text,
                style: next_style,
            };
            flush(paint, row, Some(single));
        } else {
            append_run(&mut run, col, &text, next_style);
        }
    }
    flush(paint, row, run);
}

fn append_run(run: &mut Option<Run>, col: usize, text: &str, style: Style) {
    match run.as_mut() {
        Some(active) => active.text.push_str(text),
        None => {
            *run = Some(Run {
                col,
                text: text.to_owned(),
                style,
            })
        }
    }
}

fn cell_text(cell: &Cell, line: Option<&Row>, col: usize) -> String {
    let mut text = String::from(if cell.ch == NUL_CHAR { ' ' } else { cell.ch });
    if let Some(line) = line {
        text.extend(line.marks(col).chars());
    }
    text
}

fn row_links(links: &[LinkSpan], row: isize) -> &[LinkSpan] {
    let start = links.partition_point(|span| span.row < row);
    let end = links.partition_point(|span| span.row <= row);
    &links[start..end]
}

fn style(paint: &Paint, cell: &Cell, under_cursor: bool, linked: bool) -> Style {
    let (fg, _) = colors(paint.theme, cell);
    Style {
        color: if under_cursor {
            paint.theme.terminal
        } else {
            fg
        },
        bold: cell.flags.contains(Flags::BOLD),
        italic: cell.flags.contains(Flags::ITALIC),
        underline: linked || cell.flags.intersects(Flags::ANY_UNDERLINE),
        strike: cell.flags.contains(Flags::STRIKE),
    }
}

fn flush(paint: &Paint, row: usize, run: Option<Run>) {
    let Some(run) = run else {
        return;
    };
    if run.text.trim().is_empty() && !run.style.underline && !run.style.strike {
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
    let mut desc = paint.metrics.font.clone();
    if style.bold {
        desc.set_weight(pango::Weight::Bold);
    }
    if style.italic {
        desc.set_style(pango::Style::Italic);
    }
    desc
}

fn attributes(style: Style) -> pango::AttrList {
    let list = pango::AttrList::new();
    if style.underline {
        list.insert(pango::AttrInt::new_underline(pango::Underline::Single));
    }
    if style.strike {
        list.insert(pango::AttrInt::new_strikethrough(true));
    }
    list
}
