use trame::geometry::Point;
use trame::session::SessionId;

use super::clicks::CLICK_KINDS;
use super::{Cell, Running};

impl Running {
    pub(super) fn begin_selection(&mut self, id: SessionId, cell: Cell) {
        let count = self.selecting.count_click(cell);
        let extend = self.modifiers.shift_key() && count == 1;
        if let Some(session) = self.sessions.get(id) {
            let mut terminal = session.terminal();
            let screen = terminal.screen_mut();
            if extend && screen.selection().is_some() {
                screen.extend_selection(cell.0, cell.1);
            } else {
                screen.start_selection(CLICK_KINDS[count - 1], cell.0, cell.1);
            }
        }
        self.selecting.drag = Some(id);
    }

    pub(super) fn drag_selection(&mut self, id: SessionId, point: Point) {
        let Some(cell) = self.cell_in_session(id, point) else {
            return;
        };
        if self.selecting.cell.replace(cell) == Some(cell) {
            return;
        }
        if let Some(session) = self.sessions.get(id) {
            session
                .terminal()
                .screen_mut()
                .extend_selection(cell.0, cell.1);
        }
        self.window.request_redraw();
    }
}
