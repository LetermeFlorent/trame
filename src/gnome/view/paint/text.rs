use gtk::prelude::*;
use gtk::{graphene, pango};
use trame::term::grid::Row;
use trame::term::{Cell, Flags};
use trame::ui::terminal::CursorPlace;
use trame::ui::terminal::links::LinkSpan;

use super::Paint;
use super::strips::RowCells;
use super::style::{Style, attributes, font, style};
use crate::gnome::view::theme::rgba;

const NUL_CHAR: char = '\0';

struct Run {
    col: usize,
    text: String,
    style: Style,
}

pub fn draw_row(paint: &Paint, item: &RowCells, line: Option<&Row>, block: Option<CursorPlace>) {
    let links = row_links(paint.links, item.row);
    let mut run: Option<Run> = None;
    for (col, cell) in item.cells.iter().enumerate() {
        if cell.flags.contains(Flags::WIDE_SPACER) {
            continue;
        }
        if cell.flags.contains(Flags::IMAGE) || cell.flags.contains(Flags::HIDDEN) {
            flush(paint, item.y, run.take());
            continue;
        }
        let under = usize::try_from(item.row)
            .ok()
            .is_some_and(|r| block.is_some_and(|place| place.covers(r, col)));
        let linked = links.iter().any(|span| span.covers(col));
        let next_style = style(paint, cell, under, linked);
        let text = cell_text(cell, line, col);
        let alone = !text.is_ascii() || cell.flags.contains(Flags::WIDE);
        if !run.as_ref().is_some_and(|cur| cur.style == next_style) || alone {
            flush(paint, item.y, run.take());
        }
        if alone {
            flush(
                paint,
                item.y,
                Some(Run {
                    col,
                    text,
                    style: next_style,
                }),
            );
        } else {
            append_run(&mut run, col, &text, next_style);
        }
    }
    flush(paint, item.y, run);
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

fn flush(paint: &Paint, y: f32, run: Option<Run>) {
    let Some(run) = run else { return };
    if run.text.trim().is_empty()
        && run.style.underline == pango::Underline::None
        && !run.style.strike
    {
        return;
    }
    let layout = paint.view.create_pango_layout(Some(&run.text));
    layout.set_font_description(Some(&font(paint, run.style)));
    layout.set_attributes(Some(&attributes(run.style)));
    let origin = paint.cell_y(y, run.col, 1);
    paint.snapshot.save();
    paint
        .snapshot
        .translate(&graphene::Point::new(origin.x(), origin.y()));
    paint
        .snapshot
        .append_layout(&layout, &rgba(run.style.color));
    paint.snapshot.restore();
}
