use crate::term::modes::Modes;
use crate::term::screen::Screen;
use crate::term::vt::Params;

const PRIMARY_ATTRIBUTES: &[u8] = b"\x1b[?62;22c";
const SECONDARY_ATTRIBUTES: &[u8] = b"\x1b[>1;10;0c";
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
            ([], b't') if raw == 18 => {
                let report = format!("\x1b[8;{};{}t", self.rows(), self.cols());
                self.respond(report.as_bytes());
            }
            _ => {}
        }
    }

    fn report_status(&mut self, kind: u16, private: bool) {
        match kind {
            5 => self.respond(b"\x1b[0n"),
            6 => {
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
            4 => Modes::INSERT,
            20 => Modes::NEW_LINE,
            _ => return 0,
        };
        if self.modes.contains(flag) { 1 } else { 2 }
    }
}
