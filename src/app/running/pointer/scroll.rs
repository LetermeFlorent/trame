use winit::event::MouseScrollDelta;

use trame::input::{WheelAction, arrow_keys, wheel_action};
use trame::session::SessionId;

use crate::app::running::Running;

const WHEEL_LINES: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app::running) enum Scroll {
    Pages(isize),
    Top,
    Bottom,
}

impl Running {
    pub(in crate::app::running) fn scroll_active(&mut self, scroll: Scroll) {
        let Some(session) = self
            .workspace
            .active_tab()
            .and_then(|id| self.sessions.get(id))
        else {
            return;
        };
        let id = session.id();
        let mut terminal = session.terminal();
        let screen = terminal.screen_mut();
        match scroll {
            Scroll::Pages(pages) => {
                let page = screen.rows().saturating_sub(1).max(1) as isize;
                let before = screen.grid().display_offset();
                screen.scroll_display(pages * page);
                let after = screen.grid().display_offset();
                let rows = screen.rows() as f32;
                drop(terminal);
                self.motion
                    .scroll_by(id, before as f32 - after as f32, rows);
            }
            Scroll::Top => {
                let history = screen.grid().history_len() as isize;
                screen.scroll_display(history);
                self.motion.stop_scroll(id);
            }
            Scroll::Bottom => {
                screen.reset_display();
                self.motion.stop_scroll(id);
            }
        }
    }

    pub(in crate::app::running) fn wheel(&mut self, delta: MouseScrollDelta) {
        if self.frame.blocked {
            return;
        }
        let lines = match delta {
            MouseScrollDelta::LineDelta(_, y) => y * WHEEL_LINES,
            MouseScrollDelta::PixelDelta(position) => {
                position.y as f32 / self.text.cell_metrics().height
            }
        };
        self.pointer.wheel += lines;
        let whole = self.pointer.wheel.trunc();
        self.pointer.wheel -= whole;
        if let Some(id) = self.session_under_pointer() {
            self.scroll_session(id, whole as isize);
        }
    }

    fn session_under_pointer(&self) -> Option<SessionId> {
        let point = self.pointer.position?;
        self.frame
            .panes
            .iter()
            .find(|pane| pane.body.contains(point))
            .and_then(|pane| pane.active)
    }

    fn scroll_session(&mut self, id: SessionId, lines: isize) {
        let Some(session) = self.sessions.get(id) else {
            return;
        };
        let mut terminal = session.terminal();
        let modes = terminal.screen().modes();
        match wheel_action(lines, modes) {
            WheelAction::History(lines) => {
                let screen = terminal.screen_mut();
                let before = screen.grid().display_offset();
                screen.scroll_display(lines);
                let after = screen.grid().display_offset();
                let rows = screen.rows() as f32;
                drop(terminal);
                self.motion
                    .scroll_by(id, before as f32 - after as f32, rows);
                self.window.request_redraw();
            }
            WheelAction::Keys(count, up) => {
                drop(terminal);
                session.write(arrow_keys(count, up, modes));
            }
            WheelAction::Report => {
                let bytes = self.wheel_report(id, lines, modes);
                drop(terminal);
                session.write(bytes);
            }
            WheelAction::Ignore => {}
        }
    }
}
