use trame::geometry::Point;
use trame::session::SessionId;
use trame::ui::frame::PaneFrame;

use super::{Cell, Running};

impl Running {
    pub(super) fn cell_in_session(&self, id: SessionId, point: Point) -> Option<Cell> {
        let pane = self.pane_showing(id)?;
        Some(self.cell_in_pane(pane, point))
    }

    pub(super) fn cell_in_pane(&self, pane: &PaneFrame, point: Point) -> Cell {
        let metrics = self.text.cell_metrics();
        let (rows, cols) = pane.grid_size(&metrics);
        let col = ((point.x - pane.terminal.x) / metrics.width)
            .floor()
            .max(0.0) as usize;
        let row = ((point.y - pane.terminal.y) / metrics.height)
            .floor()
            .max(0.0) as usize;
        (row.min(rows - 1), col.min(cols - 1))
    }
}
