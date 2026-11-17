use crate::term::graphics::{Command, Target};
use crate::term::screen::Screen;

impl Screen {
    pub(in crate::term::dispatch) fn delete_images(&mut self, command: &Command) {
        let id = self.graphics_id(command);
        let cursor = (self.cursor.row, self.cursor.col);
        let Some(target) = Target::of(command, id, cursor) else {
            return;
        };
        let mut touched = self.remove_placements(target);
        if command.delete.is_ascii_uppercase() {
            touched.extend(
                self.graphics
                    .ids()
                    .into_iter()
                    .filter(|&image| target.names(image)),
            );
            self.graphics
                .collect(touched, [&self.primary, &self.alternate]);
        }
    }

    pub(in crate::term::dispatch) fn remove_placements(&mut self, target: Target) -> Vec<u32> {
        let doomed = |at, anchor: &_| target.hits(at, anchor);
        self.graphics
            .virtuals
            .retain(|&image, _| !target.names(image));
        if !target.reaches_history() {
            return self.grid_mut().remove_anchors(false, doomed);
        }
        let mut touched = self.primary.remove_anchors(true, doomed);
        touched.extend(self.alternate.remove_anchors(true, doomed));
        touched
    }
}
