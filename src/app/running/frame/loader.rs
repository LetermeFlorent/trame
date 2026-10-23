use std::time::{Duration, Instant};

use trame::ui::chrome::{LoaderProgress, LoaderView};
use trame::ui::motion::ease_out;

use crate::app::running::Running;
use crate::app::update::UpdateStatus;

const FADE: Duration = Duration::from_millis(220);
const FRAME: Duration = Duration::from_millis(33);
const FULL: u8 = 100;

struct Progress {
    fraction: f32,
    label: String,
}

impl Progress {
    fn new(percent: u8) -> Self {
        Self {
            fraction: f32::from(percent) / f32::from(FULL),
            label: format!("{percent}%"),
        }
    }
}

pub(in crate::app::running) struct Loader {
    started: Instant,
    message: String,
    progress: Option<Progress>,
    closing: Option<Instant>,
    next_frame: Instant,
}

impl Loader {
    pub(in crate::app::running) fn view(&self, now: Instant) -> LoaderView<'_> {
        LoaderView {
            message: &self.message,
            progress: self.progress.as_ref().map(|progress| LoaderProgress {
                fraction: progress.fraction,
                label: &progress.label,
            }),
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
        let strings = self.strings;
        let (message, progress) = match status {
            UpdateStatus::Checking => (strings.checking_updates.to_owned(), None),
            UpdateStatus::Downloading { version, percent } => (
                format!("{} {version}", strings.downloading_version),
                Some(Progress::new(*percent)),
            ),
            UpdateStatus::Installing(version) => (
                format!("{} {version}", strings.installing_version),
                Some(Progress::new(FULL)),
            ),
            UpdateStatus::Done | UpdateStatus::Installed(_) | UpdateStatus::Available(_) => {
                return self.close_loader();
            }
        };
        let now = Instant::now();
        let loader = self.loader.get_or_insert_with(|| Loader {
            started: now,
            message: String::new(),
            progress: None,
            closing: None,
            next_frame: now,
        });
        loader.message = message;
        loader.progress = progress;
        self.window.request_redraw();
    }

    pub(in crate::app::running) fn loader_wake(&mut self, now: Instant) -> Option<Instant> {
        let loader = self.loader.as_mut()?;
        if now >= loader.next_frame && !self.visibility.is_hidden(&self.window) {
            loader.next_frame = now + FRAME;
            self.window.request_redraw();
        }
        Some(loader.next_frame)
    }

    fn close_loader(&mut self) {
        let Some(loader) = self.loader.as_mut() else {
            return;
        };
        loader.closing.get_or_insert_with(Instant::now);
        self.window.request_redraw();
    }
}
