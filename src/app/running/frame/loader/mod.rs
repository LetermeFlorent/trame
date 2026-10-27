#[cfg(test)]
mod tests;
mod view;

use std::time::{Duration, Instant};

use crate::app::running::Running;
use crate::app::update::UpdateStatus;
use view::FULL;
pub(in crate::app::running) use view::Loader;

const STILL_FRAME: Duration = Duration::from_millis(33);
const MOVING_FRAME: Duration = Duration::from_millis(16);

impl Running {
    pub fn set_update(&mut self, status: &UpdateStatus) {
        let strings = self.strings;
        let (message, percent) = match status {
            UpdateStatus::Checking => (strings.checking_updates.to_owned(), None),
            UpdateStatus::Downloading { version, percent } => (
                format!("{} {version}", strings.downloading_version),
                Some(*percent),
            ),
            UpdateStatus::Installing(version) => (
                format!("{} {version}", strings.installing_version),
                Some(FULL),
            ),
            UpdateStatus::Done
            | UpdateStatus::Installed(_)
            | UpdateStatus::Available(_)
            | UpdateStatus::Ready { .. } => {
                return self.close_loader();
            }
        };
        let now = Instant::now();
        let loader = self.loader.get_or_insert_with(|| Loader::new(now));
        loader.show(message, percent, now);
        self.window.request_redraw();
    }

    pub(in crate::app::running) fn loader_wake(&mut self, now: Instant) -> Option<Instant> {
        let loader = self.loader.as_mut()?;
        if now >= loader.next_frame && !self.visibility.is_hidden(&self.window) {
            let pace = match loader.is_moving(now) {
                true => MOVING_FRAME,
                false => STILL_FRAME,
            };
            loader.next_frame = now + pace;
            self.window.request_redraw();
        }
        Some(loader.next_frame)
    }

    fn close_loader(&mut self) {
        let Some(loader) = self.loader.as_mut() else {
            return;
        };
        loader.close(Instant::now());
        self.window.request_redraw();
    }
}
