use trame::files::open_link;
use trame::geometry::Point;
use trame::session::SessionId;
use trame::ui::Platform;
use trame::ui::terminal::links::link_at;
use trame::workspace::PaneId;

use super::{Cell, Running};

impl Running {
    pub(super) fn open_link_at(&self, id: SessionId, cell: Cell) -> bool {
        let Some(target) = self.link_target(id, cell) else {
            return false;
        };
        open_link(target);
        true
    }

    pub(in crate::app::running::pointer) fn link_under(&self, pane: PaneId, point: Point) -> bool {
        let Some(pane) = self.frame.pane(pane) else {
            return false;
        };
        let Some(id) = pane.active else {
            return false;
        };
        let reports = self.reports_mouse(id);
        !reports
            && self
                .link_target(id, self.cell_in_pane(pane, point))
                .is_some()
    }

    fn link_target(&self, id: SessionId, cell: Cell) -> Option<String> {
        if !self.preferences.detect_links || !self.primary_held() {
            return None;
        }
        let row = isize::try_from(cell.0).unwrap_or(isize::MAX);
        let session = self.sessions.get(id)?;
        link_at(session.terminal().screen(), row, cell.1)
    }

    fn primary_held(&self) -> bool {
        match self.metrics.platform {
            Platform::Mac => self.modifiers.super_key(),
            _ => self.modifiers.control_key(),
        }
    }
}
