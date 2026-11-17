use super::put_cell;
use crate::term::cell::Cell;
use crate::term::cursor::charset::Charset;
use crate::term::modes::Modes;
use crate::term::screen::Screen;

const ASCII_WIDTH: usize = 1;

impl Screen {
    pub(crate) fn print_ascii_run(&mut self, text: &[u8]) {
        let Some(&last) = text.last() else {
            return;
        };
        if !self.prints_ascii_as_is() {
            text.iter().for_each(|&byte| self.print_char(byte.into()));
            return;
        }
        self.last_char = Some(last.into());
        let (cols, template) = (self.cols(), self.print_template());
        let mut rest = text;
        while !rest.is_empty() {
            if self.cursor.pending_wrap {
                self.wrap_line();
            }
            let (row, col) = (self.cursor.row, self.cursor.col);
            let (fits, tail) = rest.split_at(rest.len().min(cols.saturating_sub(col).max(1)));
            let line = self.grid_mut().row_mut(row);
            for (at, &byte) in (col..).zip(fits) {
                put_cell(line, at, byte.into(), ASCII_WIDTH, &template);
            }
            let next = col + fits.len();
            self.cursor.col = next.min(cols - 1);
            self.cursor.pending_wrap = next >= cols;
            rest = tail;
        }
    }

    #[doc(hidden)]
    pub fn prints_ascii_as_is(&self) -> bool {
        self.modes.contains(Modes::AUTOWRAP)
            && !self.modes.contains(Modes::INSERT)
            && self.charsets.slots[self.charsets.active] == Charset::Ascii
    }

    #[doc(hidden)]
    pub fn template_sets_attributes(&self) -> bool {
        self.cursor.template != Cell::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::term::screen::Screen;

    const RUN: &[u8] = b"abc";

    #[test]
    fn a_run_from_past_the_last_column_prints_like_one_char_at_a_time() {
        let screen = Screen::new(2, 4, 0);
        for col in [screen.cols(), screen.cols() + 1] {
            let mut fast = screen.clone();
            fast.cursor.col = col;
            let mut slow = fast.clone();
            fast.print_ascii_run(RUN);
            RUN.iter().for_each(|&byte| slow.print_char(byte.into()));
            assert_eq!(
                format!("{fast:?}"),
                format!("{slow:?}"),
                "cursor at col {col}"
            );
        }
    }
}
