use crate::term::graphics::Target;
use crate::term::screen::Screen;

impl Screen {
    pub(in crate::term::dispatch) fn clear_screen_images(&mut self) {
        self.grid_mut().remove_anchors(false, |_, _| true);
    }

    pub(in crate::term::dispatch) fn clear_history_images(&mut self) {
        let gone: Vec<u32> = self
            .primary
            .history_anchors()
            .map(|anchor| anchor.image)
            .collect();
        self.primary.clear_history();
        self.graphics
            .collect(gone, [&self.primary, &self.alternate]);
    }

    pub(in crate::term::dispatch) fn replace_image(&mut self, id: u32) {
        if self.graphics.get(id).is_some() {
            self.remove_placements(Target::Image(id, 0));
        }
    }
}
