use super::codes::{MODE_RESET, MODE_SET, MODE_UNKNOWN, ansi_mode, private_mode as private};
use crate::term::modes::Modes;
use crate::term::screen::Screen;
use crate::term::vt::Params;

impl Screen {
    pub(crate) fn set_ansi_modes(&mut self, params: &Params, enabled: bool) {
        for group in params.iter() {
            match group[0] {
                ansi_mode::INSERT => self.modes.set(Modes::INSERT, enabled),
                ansi_mode::NEW_LINE => self.modes.set(Modes::NEW_LINE, enabled),
                _ => {}
            }
        }
    }

    pub(crate) fn set_private_modes(&mut self, params: &Params, enabled: bool) {
        for group in params.iter() {
            self.set_private_mode(group[0], enabled);
        }
    }

    fn set_private_mode(&mut self, mode: u16, enabled: bool) {
        match mode {
            private::ORIGIN => {
                self.modes.set(Modes::ORIGIN, enabled);
                self.goto(0, 0);
            }
            private::ALT_SCREEN_LEGACY | private::ALT_SCREEN => self.switch_screen(enabled),
            private::SAVE_CURSOR => self.save_or_restore(enabled),
            private::ALT_SCREEN_AND_CURSOR => {
                if enabled {
                    self.save_cursor();
                    self.switch_screen(true);
                } else {
                    self.switch_screen(false);
                    self.restore_cursor();
                }
            }
            private::MOUSE_CLICK | private::MOUSE_DRAG | private::MOUSE_MOTION => {
                self.modes.set(Modes::MOUSE_ANY, false);
                if let Some(flag) = simple_private_mode(mode) {
                    self.modes.set(flag, enabled);
                }
            }
            _ => {
                if let Some(flag) = simple_private_mode(mode) {
                    self.modes.set(flag, enabled);
                }
            }
        }
    }

    pub(crate) fn forget_mouse(&mut self) {
        self.modes.set(Modes::MOUSE_ANY, false);
        self.modes.set(Modes::MOUSE_SGR, false);
    }

    fn save_or_restore(&mut self, save: bool) {
        if save {
            self.save_cursor();
        } else {
            self.restore_cursor();
        }
    }

    fn switch_screen(&mut self, alternate: bool) {
        if alternate == self.alt_active() {
            return;
        }
        self.modes.set(Modes::ALT_SCREEN, alternate);
        self.alternate.reset();
        self.primary.reset_display();
        self.scroll_top = 0;
        self.scroll_bottom = self.rows() - 1;
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn private_mode_state(&self, mode: u16) -> u8 {
        let flag = match mode {
            private::ALT_SCREEN_LEGACY | private::ALT_SCREEN | private::ALT_SCREEN_AND_CURSOR => {
                Modes::ALT_SCREEN
            }
            _ => match simple_private_mode(mode) {
                Some(flag) => flag,
                None => return MODE_UNKNOWN,
            },
        };
        if self.modes.contains(flag) {
            MODE_SET
        } else {
            MODE_RESET
        }
    }
}

pub(crate) fn simple_private_mode(mode: u16) -> Option<Modes> {
    Some(match mode {
        private::APP_CURSOR => Modes::APP_CURSOR,
        private::ORIGIN => Modes::ORIGIN,
        private::AUTOWRAP => Modes::AUTOWRAP,
        private::CURSOR_VISIBLE => Modes::CURSOR_VISIBLE,
        private::MOUSE_CLICK => Modes::MOUSE_CLICK,
        private::MOUSE_DRAG => Modes::MOUSE_DRAG,
        private::MOUSE_MOTION => Modes::MOUSE_MOTION,
        private::FOCUS_EVENTS => Modes::FOCUS_EVENTS,
        private::MOUSE_SGR => Modes::MOUSE_SGR,
        private::ALTERNATE_SCROLL => Modes::ALTERNATE_SCROLL,
        private::BRACKETED_PASTE => Modes::BRACKETED_PASTE,
        private::SYNC_UPDATE => Modes::SYNC_UPDATE,
        private::WIN32_INPUT => Modes::WIN32_INPUT,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::term::Terminal;

    #[test]
    fn leaving_the_alternate_screen_frees_its_cells() {
        let mut terminal = Terminal::new(4, 20, 0);
        terminal.feed(b"\x1b[?1049hfull screen app\x1b[42m\x1b[2J");
        assert!(!terminal.screen().alternate.row(0).is_empty());
        terminal.feed(b"\x1b[?1049l");
        let alternate = &terminal.screen().alternate;
        assert!((0..alternate.rows()).all(|row| alternate.row(row).is_empty()));
    }
}
