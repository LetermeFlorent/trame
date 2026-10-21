use crate::term::cell::Cell;
use crate::term::modes::Modes;
use crate::term::screen::Screen;

impl Screen {
    pub(crate) fn esc(&mut self, intermediates: &[u8], byte: u8) {
        match (intermediates, byte) {
            ([], b'7') => self.save_cursor(),
            ([], b'8') => self.restore_cursor(),
            ([], b'D') => self.index(),
            ([], b'E') => {
                self.index();
                self.carriage_return();
            }
            ([], b'M') => self.reverse_index(),
            ([], b'H') => self.set_tab_stop(),
            ([], b'c') => self.full_reset(),
            ([], b'=') => self.modes.set(Modes::APP_KEYPAD, true),
            ([], b'>') => self.modes.set(Modes::APP_KEYPAD, false),
            ([b'('], designator) => self.charsets.designate(0, designator),
            ([b')'], designator) => self.charsets.designate(1, designator),
            ([b'#'], b'8') => self.fill_with_test_pattern(),
            _ => {}
        }
    }

    fn fill_with_test_pattern(&mut self) {
        let (rows, cols) = (self.rows(), self.cols());
        for row in 0..rows {
            let line = self.grid_mut().row_mut(row);
            for col in 0..cols {
                *line.cell_mut(col) = Cell {
                    ch: 'E',
                    ..Cell::default()
                };
            }
        }
        self.goto(0, 0);
    }
}
