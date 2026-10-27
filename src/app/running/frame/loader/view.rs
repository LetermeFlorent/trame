use std::time::{Duration, Instant};

use trame::ui::chrome::{LoaderProgress, LoaderView};
use trame::ui::motion::ease_out;

const ENTER: Duration = Duration::from_millis(320);
const FADE: Duration = Duration::from_millis(350);
const BLEND: Duration = Duration::from_millis(220);
const GLIDE: Duration = Duration::from_millis(400);
const RISE: f32 = 8.0;
const DROP: f32 = 6.0;
pub(super) const FULL: u8 = 100;

struct Progress {
    percent: u8,
    from: f32,
    since: Instant,
    label: String,
}

impl Progress {
    fn toward(before: Option<Self>, percent: u8, now: Instant) -> Self {
        match before {
            Some(before) if before.percent == percent => before,
            before => Self {
                percent,
                from: before.map_or_else(|| target(percent), |before| before.fraction(now)),
                since: now,
                label: format!("{percent}%"),
            },
        }
    }

    fn fraction(&self, now: Instant) -> f32 {
        let to = target(self.percent);
        self.from + (to - self.from) * ease_out(share(self.since, now, GLIDE))
    }
}

pub(in crate::app::running) struct Loader {
    started: Instant,
    message: String,
    previous: Option<(String, Instant)>,
    progress: Option<Progress>,
    closing: Option<Instant>,
    pub(super) next_frame: Instant,
}

impl Loader {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            started: now,
            message: String::new(),
            previous: None,
            progress: None,
            closing: None,
            next_frame: now,
        }
    }

    pub(super) fn show(&mut self, message: String, percent: Option<u8>, now: Instant) {
        if message != self.message {
            let old = std::mem::replace(&mut self.message, message);
            self.previous = (!old.is_empty()).then_some((old, now));
        }
        let before = self.progress.take();
        self.progress = percent.map(|percent| Progress::toward(before, percent, now));
    }

    pub(super) fn close(&mut self, now: Instant) {
        self.closing.get_or_insert(now);
    }

    pub(in crate::app::running) fn view(&self, now: Instant) -> LoaderView<'_> {
        let entered = ease_out(share(self.started, now, ENTER));
        let gone = self
            .closing
            .map_or(0.0, |closing| ease_out(share(closing, now, FADE)));
        let previous = self.previous.as_ref();
        LoaderView {
            message: &self.message,
            previous: previous.map(|(message, _)| message.as_str()),
            blend: previous.map_or(1.0, |(_, since)| ease_out(share(*since, now, BLEND))),
            progress: self.progress.as_ref().map(|progress| LoaderProgress {
                fraction: progress.fraction(now),
                label: &progress.label,
            }),
            spin: now.saturating_duration_since(self.started).as_secs_f32(),
            opacity: 1.0 - gone,
            content: entered,
            shift: RISE * (1.0 - entered) + DROP * gone,
        }
    }

    pub(in crate::app::running) fn is_gone(&self, now: Instant) -> bool {
        self.closing
            .is_some_and(|closing| share(closing, now, FADE) >= 1.0)
    }

    pub(super) fn is_moving(&self, now: Instant) -> bool {
        let before = |since, span| share(since, now, span) < 1.0;
        let blending = self
            .previous
            .as_ref()
            .is_some_and(|(_, since)| before(*since, BLEND));
        let gliding = self
            .progress
            .as_ref()
            .is_some_and(|progress| before(progress.since, GLIDE));
        before(self.started, ENTER) || blending || gliding || self.closing.is_some()
    }
}

fn target(percent: u8) -> f32 {
    f32::from(percent) / f32::from(FULL)
}

fn share(since: Instant, now: Instant, span: Duration) -> f32 {
    now.saturating_duration_since(since).as_secs_f32() / span.as_secs_f32()
}
