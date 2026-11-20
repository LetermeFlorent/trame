mod cells;
mod clicks;
mod clipboard;
mod drag;
mod links;
mod pictures;
mod report;

pub(super) use links::LinkHover;
pub(in crate::app::running) use pictures::{HoverPreview, PictureView};

use std::time::Instant;

use winit::event::{ElementState, MouseButton as Button};

use trame::geometry::Point;
use trame::input::{MouseAction, MouseButton};
use trame::session::SessionId;
use trame::ui::Hit;

use crate::app::running::Running;

type Cell = (usize, usize);

#[derive(Default)]
pub(in crate::app::running) struct Selecting {
    drag: Option<SessionId>,
    held: Option<(SessionId, MouseButton)>,
    cell: Option<Cell>,
    clicks: Option<(Instant, Cell, usize)>,
}

impl Selecting {
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }

    #[cfg(test)]
    pub(super) fn is_clear(&self) -> bool {
        self.drag.is_none() && self.held.is_none() && self.cell.is_none() && self.clicks.is_none()
    }
}

impl Running {
    pub(in crate::app::running) fn terminal_button(
        &mut self,
        state: ElementState,
        button: Button,
    ) -> bool {
        if self.picture_active() {
            return true;
        }
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
        if self.picture_active() {
            return;
        }
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
        if !reports && button != MouseButton::Left {
            return false;
        }
        self.desk.focus_pane(pane_id);
        if pictures::picture_action(false, button, reports, self.primary_held())
            == pictures::PictureAction::Open
            && self.open_picture_at(id, cell)
        {
            return true;
        }
        if !reports && self.open_link_at(id, cell) {
            self.changed();
            return true;
        }
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
}
