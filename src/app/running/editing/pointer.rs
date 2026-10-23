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
        let pressed = state == ElementState::Pressed;
        match hit {
            Some(Hit::EditSwatch(color)) if pressed => self.pick_color(color),
            Some(Hit::EditEmblem(emblem)) if pressed => self.pick_emblem(emblem),
            Some(Hit::EditField | Hit::EditStrip | Hit::EditSwatch(_) | Hit::EditEmblem(_)) => {
                return true;
            }
            _ if pressed => {
                self.cancel_edit();
                self.changed();
                return false;
            }
            _ => return false,
        }
        self.changed();
        true
    }
}
