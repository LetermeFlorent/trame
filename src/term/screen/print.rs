use unicode_width::UnicodeWidthChar;

use super::Screen;
use crate::term::cell::{Cell, Flags};
use crate::term::grid::Row;
use crate::term::modes::Modes;

impl Screen {
    pub(crate) fn print_char(&mut self, c: char) {
        let c = self.charsets.map(c);
        let width = c.width().unwrap_or(0);
        if width == 0 {
            return;
        }
        self.last_char = Some(c);
        let cols = self.cols();
        let autowrap = self.modes.contains(Modes::AUTOWRAP);
        if self.cursor.pending_wrap && autowrap {
            self.wrap_line();
        }
        if width == 2 && self.cursor.col + 1 >= cols {
            if !autowrap || cols < 2 {
                return;
            }
            self.wrap_line();
        }
        if self.modes.contains(Modes::INSERT) {
            let (row, col, template) = (self.cursor.row, self.cursor.col, self.cursor.template);
            self.grid_mut()
                .row_mut(row)
                .insert_blanks(col, width, cols, &template);
        }
        self.write_cell(c, width);
        let next = self.cursor.col + width;
        if next >= cols {
            self.cursor.col = cols - 1;
            self.cursor.pending_wrap = autowrap;
        } else {
            self.cursor.col = next;
        }
    }

    pub(crate) fn repeat_last(&mut self, count: usize) {
        let Some(c) = self.last_char else {
            return;
        };
        let count = count.min(self.cols() * self.rows());
        for _ in 0..count {
            self.print_char(c);
        }
    }

    fn wrap_line(&mut self) {
        let row = self.cursor.row;
        self.grid_mut().row_mut(row).wrapped = true;
        self.cursor.col = 0;
        self.cursor.pending_wrap = false;
        self.index();
    }

    fn write_cell(&mut self, c: char, width: usize) {
        let (row, col, template) = (self.cursor.row, self.cursor.col, self.cursor.template);
        let line = self.grid_mut().row_mut(row);
        clear_wide_fragments(line, col, width);
        let cell = line.cell_mut(col);
        *cell = Cell { ch: c, ..template };
        cell.flags.remove(Flags::WIDTH_MARKS);
        if width == 2 {
            cell.flags.insert(Flags::WIDE);
            let spacer = line.cell_mut(col + 1);
            *spacer = Cell {
                ch: ' ',
                ..template
            };
            spacer.flags.remove(Flags::WIDTH_MARKS);
            spacer.flags.insert(Flags::WIDE_SPACER);
        }
    }
}

fn clear_wide_fragments(line: &mut Row, col: usize, width: usize) {
    if col > 0 && line.cell(col).flags.contains(Flags::WIDE_SPACER) {
        let leader = line.cell_mut(col - 1);
        leader.ch = ' ';
        leader.flags.remove(Flags::WIDTH_MARKS);
    }
    let last = col + width - 1;
    if line.cell(last).flags.contains(Flags::WIDE) {
        let spacer = line.cell_mut(last + 1);
        spacer.ch = ' ';
        spacer.flags.remove(Flags::WIDTH_MARKS);
    }
}
