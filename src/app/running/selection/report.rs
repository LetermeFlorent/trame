use trame::geometry::Point;
use trame::input::{MouseAction, MouseButton, MouseEvent, MouseModifiers, encode_mouse};
use trame::session::SessionId;
use trame::term::Modes;

use super::{Cell, Running};

const MAX_WHEEL_REPORTS: usize = 10;

impl Running {
    pub(in crate::app::running) fn wheel_report(
        &self,
        id: SessionId,
        lines: isize,
        modes: Modes,
    ) -> Vec<u8> {
        let Some(cell) = self
            .pointer
            .position
            .and_then(|point| self.cell_in_session(id, point))
        else {
            return Vec::new();
        };
        let button = if lines > 0 {
            MouseButton::WheelUp
        } else {
            MouseButton::WheelDown
        };
        let event = self.mouse_event(MouseAction::Press, Some(button), cell);
        let report = encode_mouse(&event, modes).unwrap_or_default();
        report.repeat(lines.unsigned_abs().min(MAX_WHEEL_REPORTS))
    }

    pub(super) fn report_motion(&mut self, id: SessionId, held: Option<MouseButton>, point: Point) {
        let Some(cell) = self.cell_in_session(id, point) else {
            return;
        };
        if self.selecting.cell.replace(cell) != Some(cell) {
            self.send_report(id, MouseAction::Motion, held, cell);
        }
    }

    pub(super) fn reports_mouse(&self, id: SessionId) -> bool {
        let modes = self
            .sessions
            .get(id)
            .map(|session| session.terminal().screen().modes());
        let wanted = modes.is_some_and(|modes| modes.intersects(Modes::MOUSE_ANY));
        wanted && !self.modifiers.shift_key()
    }

    pub(super) fn send_report(
        &self,
        id: SessionId,
        action: MouseAction,
        held: Option<MouseButton>,
        cell: Cell,
    ) {
        let Some(session) = self.sessions.get(id) else {
            return;
        };
        let modes = session.terminal().screen().modes();
        let event = self.mouse_event(action, held, cell);
        if let Some(bytes) = encode_mouse(&event, modes) {
            session.write(bytes);
        }
    }

    pub(super) fn mouse_event(
        &self,
        action: MouseAction,
        button: Option<MouseButton>,
        cell: Cell,
    ) -> MouseEvent {
        let modifiers = MouseModifiers {
            shift: self.modifiers.shift_key(),
            alt: self.modifiers.alt_key(),
            ctrl: self.modifiers.control_key(),
        };
        MouseEvent {
            action,
            button,
            col: cell.1,
            row: cell.0,
            modifiers,
        }
    }
}
