use trame::desk::{Effect, Effects};

use super::Running;

impl Running {
    pub(super) fn apply(&mut self, effects: Effects) {
        for effect in effects {
            match effect {
                Effect::Redraw => self.window.request_redraw(),
                Effect::Relayout => self.relayout(),
                Effect::Changed => self.changed(),
                Effect::Rescale => self.apply_scale(),
                Effect::Animate => self.motion.animate_next(),
                Effect::Raise => self.bring_forward(),
            }
        }
    }
}
