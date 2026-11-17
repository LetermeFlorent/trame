use std::time::Instant;

use trame::theme::Rgba;

use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn sync_graphics(&mut self) {
        if self.graphics.take_fresh() {
            self.changed();
        }
        self.desk.tick(Instant::now());
        self.restart_if_card_lost();
    }

    pub(in crate::app::running) fn clear_color(&self) -> Rgba {
        self.desk.theme.chrome
    }
}
