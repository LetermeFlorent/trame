mod cells;
mod clicks;
mod clipboard;
mod report;

use std::time::Instant;

use winit::event::{ElementState, MouseButton as Button};

use trame::geometry::Point;
use trame::input::{MouseAction, MouseButton};
use trame::session::SessionId;
use trame::ui::Hit;

use crate::app::running::Running;
use clicks::CLICK_KINDS;

type Cell = (usize, usize);

#[derive(Default)]
pub(in crate::app::running) struct Selecting {
    drag: Option<SessionId>,
    held: Option<(SessionId, MouseButton)>,
    cell: Option<Cell>,
    clicks: Option<(Instant, Cell, usize)>,
}

impl Running {
    pub(in crate::app::running) fn terminal_button(
        &mut self,
        state: ElementState,
        button: Button,
    ) -> bool {
        let button = match button {
            Button::Left => MouseButton::Left,
            Button::Middle => MouseButton::Middle,
            Button::Right => MouseButton::Right,
            _ => return false,
        };
        match state {
            ElementState::Pressed => self.terminal_press(button),
            ElementState::Released => self.terminal_release(button),
        }
    }

    pub(in crate::app::running) fn terminal_motion(&mut self, point: Point) {
        if let Some(id) = self.selecting.drag {
            self.drag_selection(id, point);
            return;
        }
        if let Some((id, button)) = self.selecting.held {
            self.report_motion(id, Some(button), point);
            return;
        }
        if let Some(Hit::Terminal(pane)) = self.view.hover
            && let Some(id) = self.frame.pane(pane).and_then(|found| found.active)
            && self.reports_mouse(id)
        {
            self.report_motion(id, None, point);
        }
    }

    fn terminal_press(&mut self, button: MouseButton) -> bool {
        let Some(point) = self.pointer.position else {
            return false;
        };
        let Some(Hit::Terminal(pane_id)) = self.frame.hit(point, self.resizable()) else {
            return false;
        };
        let Some(pane) = self.frame.pane(pane_id) else {
            return false;
        };
        let Some(id) = pane.active else {
            return false;
        };
        let cell = self.cell_in_pane(pane, point);
        let reports = self.reports_mouse(id);
        if !reports && button == MouseButton::Right {
            self.focus_pane(pane_id);
            self.paste();
            self.changed();
            return true;
        }
        if !reports && button != MouseButton::Left {
            return false;
        }
        self.focus_pane(pane_id);
        self.selecting.cell = Some(cell);
        if reports {
            self.selecting.held = Some((id, button));
            self.send_report(id, MouseAction::Press, Some(button), cell);
        } else {
            self.begin_selection(id, cell);
        }
        self.changed();
        true
    }

    fn terminal_release(&mut self, button: MouseButton) -> bool {
        if button == MouseButton::Left && self.selecting.drag.take().is_some() {
            self.copy_selection();
            return true;
        }
        let Some((id, _)) = self.selecting.held.filter(|(_, held)| *held == button) else {
            return false;
        };
        self.selecting.held = None;
        if let Some(cell) = self
            .pointer
            .position
            .and_then(|point| self.cell_in_session(id, point))
        {
            self.send_report(id, MouseAction::Release, Some(button), cell);
        }
        true
    }

    fn begin_selection(&mut self, id: SessionId, cell: Cell) {
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

    fn drag_selection(&mut self, id: SessionId, point: Point) {
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
