use trame::desk::{Effect, Effects};

use super::Running;
use super::pointer::write_clipboard;

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
                Effect::StartQueued => {
                    let grids = self.grids();
                    self.desk.start_queued(&grids);
                }
                Effect::StartShown => {
                    self.start_shown();
                }
                Effect::Typed(id) => {
                    self.motion.stop_scroll(id);
                    self.window.request_redraw();
                }
                Effect::HostsChanged => self.reload_hosts(),
                Effect::Clipboard(text) => write_clipboard(&text),
                Effect::Notify { title, body } => self.notifier.show(&self.window, &title, &body),
            }
        }
    }
}
