use crate::term::modes::Modes;
use crate::term::screen::Screen;
use crate::term::vt::Params;

impl Screen {
    pub(crate) fn set_ansi_modes(&mut self, params: &Params, enabled: bool) {
        for group in params.iter() {
            match group[0] {
                4 => self.modes.set(Modes::INSERT, enabled),
                20 => self.modes.set(Modes::NEW_LINE, enabled),
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
            6 => {
                self.modes.set(Modes::ORIGIN, enabled);
                self.goto(0, 0);
            }
            47 | 1047 => self.switch_screen(enabled),
            1048 => self.save_or_restore(enabled),
            1049 => {
                if enabled {
                    self.save_cursor();
                    self.switch_screen(true);
                } else {
                    self.switch_screen(false);
                    self.restore_cursor();
                }
            }
            1000 | 1002 | 1003 => {
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
        if alternate {
            self.alternate.reset();
        }
        self.primary.reset_display();
        self.scroll_top = 0;
        self.scroll_bottom = self.rows() - 1;
        self.cursor.pending_wrap = false;
    }

    pub(crate) fn private_mode_state(&self, mode: u16) -> u8 {
        let flag = match mode {
            47 | 1047 | 1049 => Modes::ALT_SCREEN,
            _ => match simple_private_mode(mode) {
                Some(flag) => flag,
                None => return 0,
            },
        };
        if self.modes.contains(flag) { 1 } else { 2 }
    }
}

pub(crate) fn simple_private_mode(mode: u16) -> Option<Modes> {
    Some(match mode {
        1 => Modes::APP_CURSOR,
        6 => Modes::ORIGIN,
        7 => Modes::AUTOWRAP,
        25 => Modes::CURSOR_VISIBLE,
        1000 => Modes::MOUSE_CLICK,
        1002 => Modes::MOUSE_DRAG,
        1003 => Modes::MOUSE_MOTION,
        1004 => Modes::FOCUS_EVENTS,
        1006 => Modes::MOUSE_SGR,
        1007 => Modes::ALTERNATE_SCROLL,
        2004 => Modes::BRACKETED_PASTE,
        2026 => Modes::SYNC_UPDATE,
        _ => return None,
    })
}
