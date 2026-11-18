use crate::term::screen::Screen;

impl Screen {
    pub(super) fn report_cell_size(&mut self, bell_terminated: bool) {
        let (width, height) = self.cell_pixels;
        if width == 0 || height == 0 {
            return;
        }
        let reply = format!("\x1b]1337;ReportCellSize={height};{width}");
        self.respond_osc(&reply, bell_terminated);
    }
}
