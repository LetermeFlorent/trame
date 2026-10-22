mod decode;
mod erase;
mod esc;
mod osc;
mod reports;
mod set_mode;
mod sgr;

use crate::term::cursor::CursorShape;
use crate::term::screen::Screen;
use crate::vt::Params;

impl Screen {
    pub(crate) fn csi(&mut self, params: &Params, intermediates: &[u8], action: u8) {
        let count = usize::from(params.first_or(0, 1));
        let raw = params.raw(0);
        match (intermediates, action) {
            ([], b'@') => self.insert_chars(count),
            ([], b'A') => self.move_up(count),
            ([], b'B' | b'e') => self.move_down(count),
            ([], b'C' | b'a') => self.move_forward(count),
            ([], b'D') => self.move_backward(count),
            ([], b'E') => self.cursor_next_line(count),
            ([], b'F') => self.cursor_previous_line(count),
            ([], b'G' | b'`') => self.goto_col(count - 1),
            ([], b'H' | b'f') => self.cursor_position(params),
            ([], b'I') => self.tab_forward(count),
            ([] | [b'?'], b'J') => self.erase_display(raw),
            ([] | [b'?'], b'K') => self.erase_line(raw),
            ([], b'L') => self.insert_lines(count),
            ([], b'M') => self.delete_lines(count),
            ([], b'P') => self.delete_chars(count),
            ([], b'S') => self.scroll_up(count),
            ([], b'T') => self.scroll_down(count),
            ([], b'X') => self.erase_chars(count),
            ([], b'Z') => self.tab_backward(count),
            ([], b'b') => self.repeat_last(count),
            ([], b'd') => self.goto_row(count - 1),
            ([], b'g') => self.clear_tab_stops(raw),
            ([], b'h') => self.set_ansi_modes(params, true),
            ([], b'l') => self.set_ansi_modes(params, false),
            ([b'?'], b'h') => self.set_private_modes(params, true),
            ([b'?'], b'l') => self.set_private_modes(params, false),
            ([], b'm') => self.apply_sgr(params),
            ([], b'r') => self.set_scroll_region(params.raw(0).into(), params.raw(1).into()),
            ([], b's') => self.save_cursor(),
            ([], b'u') => self.restore_cursor(),
            ([b' '], b'q') => self.cursor_shape = CursorShape::from_decscusr(raw),
            ([b'!'], b'p') => self.soft_reset(),
            _ => self.report(params, intermediates, action),
        }
    }
    fn cursor_next_line(&mut self, count: usize) {
        self.move_down(count);
        self.carriage_return();
    }

    fn cursor_previous_line(&mut self, count: usize) {
        self.move_up(count);
        self.carriage_return();
    }

    fn cursor_position(&mut self, params: &Params) {
        let row = usize::from(params.first_or(0, 1)) - 1;
        let col = usize::from(params.first_or(1, 1)) - 1;
        self.goto(row, col);
    }
}
