use super::Screen;
use crate::term::modes::Modes;

const BEL: u8 = 0x07;
const BS: u8 = 0x08;
const HT: u8 = 0x09;
const LF: u8 = 0x0A;
const FF: u8 = 0x0C;
const CR: u8 = 0x0D;
const SO: u8 = 0x0E;
const SI: u8 = 0x0F;

impl Screen {
    pub(crate) fn execute_control(&mut self, byte: u8) {
        match byte {
            BEL => {
                self.attention.get_or_insert_default();
            }
            BS => self.backspace(),
            HT => self.tab_forward(1),
            LF..=FF => {
                self.index();
                if self.modes.contains(Modes::NEW_LINE) {
                    self.carriage_return();
                }
            }
            CR => self.carriage_return(),
            SO => self.charsets.active = 1,
            SI => self.charsets.active = 0,
            _ => {}
        }
    }

    pub(crate) fn index(&mut self) {
        self.cursor.pending_wrap = false;
        if self.cursor.row == self.scroll_bottom {
            self.scroll_up(1);
        } else if self.cursor.row + 1 < self.rows() {
            self.cursor.row += 1;
        }
    }

    pub(crate) fn reverse_index(&mut self) {
        self.cursor.pending_wrap = false;
        if self.cursor.row == self.scroll_top {
            self.scroll_down(1);
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
        }
    }

    pub(crate) fn carriage_return(&mut self) {
        self.cursor.col = 0;
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn scroll_up(&mut self, count: usize) {
        let (top, bottom, template) = (self.scroll_top, self.scroll_bottom, self.cursor.template);
        self.grid_mut().scroll_up(top, bottom, count, &template);
    }

    pub(crate) fn scroll_down(&mut self, count: usize) {
        let (top, bottom, template) = (self.scroll_top, self.scroll_bottom, self.cursor.template);
        self.grid_mut().scroll_down(top, bottom, count, &template);
    }

    pub(crate) fn tab_forward(&mut self, count: usize) {
        let last = self.cols() - 1;
        for _ in 0..count {
            let next = (self.cursor.col + 1..=last).find(|&col| self.tab_stops[col]);
            self.cursor.col = next.unwrap_or(last);
        }
    }

    pub(crate) fn tab_backward(&mut self, count: usize) {
        for _ in 0..count {
            let previous = (0..self.cursor.col).rev().find(|&col| self.tab_stops[col]);
            self.cursor.col = previous.unwrap_or(0);
        }
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn set_tab_stop(&mut self) {
        let col = self.cursor.col;
        self.tab_stops[col] = true;
    }

    pub(crate) fn clear_tab_stops(&mut self, mode: u16) {
        match mode {
            0 => {
                let col = self.cursor.col;
                self.tab_stops[col] = false;
            }
            3 => self.tab_stops.fill(false),
            _ => {}
        }
    }

    fn backspace(&mut self) {
        self.cursor.pending_wrap = false;
        self.cursor.col = self.cursor.col.saturating_sub(1);
    }
}
