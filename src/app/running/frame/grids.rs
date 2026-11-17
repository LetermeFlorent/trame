use trame::desk::Grids;

use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn grids(&self) -> Grids {
        let cell = self.text.cell_metrics();
        let mut grids = Grids::default();
        for pane in &self.frame.panes {
            grids.add_pane(&pane.tabs, pane.active, pane.grid_size(&cell));
        }
        grids
    }

    pub(in crate::app::running) fn start_shown(&mut self) -> bool {
        let grids = self.grids();
        self.desk.start_shown(&grids)
    }
}
