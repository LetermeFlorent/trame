use std::io::Write;

use super::codes::{MODE_RESET, MODE_SET, MODE_UNKNOWN, ansi_mode};
use crate::term::modes::Modes;
use crate::term::screen::Screen;
use crate::term::vt::Params;

const PRIMARY_ATTRIBUTES: &[u8] = b"\x1b[?62;22c";
const SECONDARY_ATTRIBUTES: &[u8] = b"\x1b[>1;10;0c";
const DEVICE_STATUS_REQUEST: u16 = 5;
const CURSOR_POSITION_REQUEST: u16 = 6;
const VERSION_REPORT: &str = concat!("\x1bP>|trame ", env!("CARGO_PKG_VERSION"), "\x1b\\");

impl Screen {
    pub(crate) fn report(&mut self, params: &Params, intermediates: &[u8], action: u8) {
        let raw = params.raw(0);
        match (intermediates, action) {
            ([], b'c') if raw == 0 => self.respond(PRIMARY_ATTRIBUTES),
            ([b'>'], b'c') => self.respond(SECONDARY_ATTRIBUTES),
            ([], b'n') => self.report_status(raw, false),
            ([b'?'], b'n') => self.report_status(raw, true),
            ([b'?', b'$'], b'p') => {
                let state = self.private_mode_state(raw);
                self.respond(format!("\x1b[?{raw};{state}$y").as_bytes());
            }
            ([b'$'], b'p') => {
                let state = self.ansi_mode_state(raw);
                self.respond(format!("\x1b[{raw};{state}$y").as_bytes());
            }
            ([b'>'], b'q') => self.respond(VERSION_REPORT.as_bytes()),
            ([], b't') => self.report_window(raw),
            _ => {}
        }
    }

    pub(super) fn respond_osc(&mut self, body: &str, bell_terminated: bool) {
        let terminator = if bell_terminated { "\x07" } else { "\x1b\\" };
        self.respond_string(&format!("{body}{terminator}"));
    }

    pub(super) fn respond_string(&mut self, sequence: &str) {
        if !self.modes.contains(Modes::WIN32_INPUT) {
            self.respond(sequence.as_bytes());
            return;
        }
        let mut pressed = Vec::new();
        for unit in sequence.encode_utf16() {
            for key_down in [1, 0] {
                let _ = write!(pressed, "\x1b[0;0;{unit};{key_down};0;1_");
            }
        }
        self.respond(&pressed);
    }

    fn report_status(&mut self, kind: u16, private: bool) {
        match kind {
            DEVICE_STATUS_REQUEST => self.respond(b"\x1b[0n"),
            CURSOR_POSITION_REQUEST => {
                let origin = self.modes.contains(Modes::ORIGIN);
                let top = if origin { self.scroll_top } else { 0 };
                let row = self.cursor.row.saturating_sub(top) + 1;
                let col = self.cursor.col + 1;
                let marker = if private { "?" } else { "" };
                self.respond(format!("\x1b[{marker}{row};{col}R").as_bytes());
            }
            _ => {}
        }
    }

    fn ansi_mode_state(&self, mode: u16) -> u8 {
        let flag = match mode {
            ansi_mode::INSERT => Modes::INSERT,
            ansi_mode::NEW_LINE => Modes::NEW_LINE,
            _ => return MODE_UNKNOWN,
        };
        if self.modes.contains(flag) {
            MODE_SET
        } else {
            MODE_RESET
        }
    }
}
