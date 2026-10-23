use std::time::{Duration, Instant};

use trame::ui::chrome::LoaderView;
use trame::ui::motion::ease_out;

use crate::app::running::Running;
use crate::app::update::UpdateStatus;

const FADE: Duration = Duration::from_millis(220);

pub(in crate::app::running) struct Loader {
    started: Instant,
    message: String,
    closing: Option<Instant>,
}

impl Loader {
    pub(in crate::app::running) fn view(&self, now: Instant) -> LoaderView<'_> {
        LoaderView {
            message: &self.message,
            spin: now.saturating_duration_since(self.started).as_secs_f32(),
            opacity: self.opacity(now),
        }
    }

    pub(in crate::app::running) fn is_gone(&self, now: Instant) -> bool {
        self.closing
            .is_some_and(|closing| now.saturating_duration_since(closing) >= FADE)
    }

    fn opacity(&self, now: Instant) -> f32 {
        self.closing.map_or(1.0, |closing| {
            let faded = now.saturating_duration_since(closing).as_secs_f32();
            1.0 - ease_out(faded / FADE.as_secs_f32())
        })
    }
}

impl Running {
    pub fn set_update(&mut self, status: &UpdateStatus) {
        let message = match status {
            UpdateStatus::Checking => self.strings.checking_updates.to_owned(),
            UpdateStatus::Downloading(version) => {
                format!("{} {version}", self.strings.downloading_version)
            }
            UpdateStatus::Done | UpdateStatus::Installed(_) => return self.close_loader(),
        };
        match &mut self.loader {
            Some(loader) => loader.message = message,
            None => {
                self.loader = Some(Loader {
                    started: Instant::now(),
                    message,
                    closing: None,
                });
            }
        }
        self.window.request_redraw();
    }

    fn close_loader(&mut self) {
        let Some(loader) = self.loader.as_mut() else {
            return;
        };
        loader.closing.get_or_insert_with(Instant::now);
        self.window.request_redraw();
    }
}
