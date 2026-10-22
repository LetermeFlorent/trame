use winit::event::ElementState;

use trame::ui::Hit;

use super::Running;

impl Running {
    pub(in super::super) fn editing_button(&mut self, state: ElementState) -> bool {
        if self.editing.is_none() {
            return false;
        }
        let hit = self
            .pointer
            .position
            .and_then(|point| self.frame.hit(point, false));
        match (hit, state) {
            (Some(Hit::EditSwatch(color)), ElementState::Pressed) => {
                self.pick_color(color);
                self.changed();
                true
            }
            (Some(Hit::EditField | Hit::EditSwatch(_)), ElementState::Released) => true,
            (Some(Hit::EditField), ElementState::Pressed) => true,
            (_, ElementState::Pressed) => {
                self.cancel_edit();
                self.changed();
                false
            }
            _ => false,
        }
    }
}
