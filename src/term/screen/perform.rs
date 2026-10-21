use super::Screen;
use crate::vt::{Params, Perform};

impl Perform for Screen {
    fn print(&mut self, c: char) {
        self.print_char(c);
    }

    fn execute(&mut self, byte: u8) {
        self.execute_control(byte);
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], action: u8) {
        self.csi(params, intermediates, action);
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], byte: u8) {
        self.esc(intermediates, byte);
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        self.osc(params, bell_terminated);
    }
}
