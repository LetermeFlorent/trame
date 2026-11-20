use std::path::PathBuf;

use trame::files::open_link;
use trame::geometry::Rect;
use trame::session::SessionId;
use trame::text::CellMetrics;
use trame::ui::terminal::links::{LinkHit, LinkKind, link_hit};
use trame::ui::{Hit, Platform};

use super::{Cell, Running};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::app::running::pointer) enum LinkHover {
    Web,
    Image(PathBuf, Rect),
}

impl Running {
    pub(super) fn open_link_at(&mut self, id: SessionId, cell: Cell) -> bool {
        let Some(hit) = self.link_target(id, cell) else {
            return false;
        };
        match hit.kind {
            LinkKind::Web => open_link(hit.target),
            LinkKind::Image => self.open_picture_path(PathBuf::from(hit.target)),
        }
        true
    }

    pub(in crate::app::running::pointer) fn hovered_link(
        &self,
        hover: Option<Hit>,
    ) -> Option<LinkHover> {
        let (Some(Hit::Terminal(pane)), Some(point)) = (hover, self.pointer.position) else {
            return None;
        };
        let pane = self.frame.pane(pane)?;
        let id = pane.active?;
        if self.reports_mouse(id) {
            return None;
        }
        let cell = self.cell_in_pane(pane, point);
        let hit = self.link_target(id, cell)?;
        Some(link_hover_from_hit(
            hit,
            pane.terminal,
            cell,
            self.text.cell_metrics(),
        ))
    }

    fn link_target(&self, id: SessionId, cell: Cell) -> Option<LinkHit> {
        if !self.desk.preferences.detect_links || !self.primary_held() {
            return None;
        }
        let row = isize::try_from(cell.0).unwrap_or(isize::MAX);
        let cwd = self.desk.sessions.cwd(id);
        let session = self.desk.sessions.get(id)?;
        link_hit(session.terminal().screen(), row, cell.1, cwd.as_deref())
    }

    pub(crate) fn primary_held(&self) -> bool {
        match self.metrics.platform {
            Platform::Mac => self.modifiers.super_key(),
            _ => self.modifiers.control_key(),
        }
    }
}

pub(in crate::app::running::pointer) fn link_hover_from_hit(
    hit: LinkHit,
    terminal: Rect,
    cell: Cell,
    metrics: CellMetrics,
) -> LinkHover {
    if hit.kind != LinkKind::Image {
        return LinkHover::Web;
    }
    let row = isize::try_from(cell.0).unwrap_or(isize::MAX);
    let Some(span) = hit
        .spans
        .iter()
        .find(|s| s.row == row && s.covers(cell.1))
        .or_else(|| hit.spans.first())
    else {
        return LinkHover::Web;
    };
    let anchor = Rect::new(
        terminal.x + span.start as f32 * metrics.width,
        terminal.y + span.row as f32 * metrics.height,
        (span.end - span.start) as f32 * metrics.width,
        metrics.height,
    );
    LinkHover::Image(PathBuf::from(hit.target), anchor)
}
