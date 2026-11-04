use trame::files::open_link;
use trame::session::SessionId;
use trame::ui::Platform;
use trame::ui::terminal::links::link_at;

use super::{Cell, Running};

impl Running {
    pub(super) fn open_link_at(&self, id: SessionId, cell: Cell) -> bool {
        if !self.preferences.detect_links || !self.primary_held() {
            return false;
        }
        let row = isize::try_from(cell.0).unwrap_or(isize::MAX);
        let target = self
            .sessions
            .get(id)
            .and_then(|session| link_at(session.terminal().screen(), row, cell.1));
        let Some(target) = target else {
            return false;
        };
        open_link(target);
        true
    }

    fn primary_held(&self) -> bool {
        match self.metrics.platform {
            Platform::Mac => self.modifiers.super_key(),
            _ => self.modifiers.control_key(),
        }
    }
}
