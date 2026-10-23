use std::time::{Duration, Instant};

use crate::geometry::Rect;

pub const DURATION: Duration = Duration::from_millis(120);

pub trait Lerp: Copy + PartialEq {
    fn lerp(self, to: Self, amount: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(self, to: Self, amount: f32) -> Self {
        self + (to - self) * amount
    }
}

impl Lerp for Rect {
    fn lerp(self, to: Self, amount: f32) -> Self {
        let edge = |from: f32, to: f32| from.lerp(to, amount).round();
        Rect::from_edges(
            edge(self.x, to.x),
            edge(self.y, to.y),
            edge(self.right(), to.right()),
            edge(self.bottom(), to.bottom()),
        )
    }
}

pub fn ease_out(progress: f32) -> f32 {
    let rest = 1.0 - progress.clamp(0.0, 1.0);
    1.0 - rest * rest * rest
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tween<T> {
    from: T,
    to: T,
    start: Instant,
}

impl<T: Lerp> Tween<T> {
    pub fn settled(value: T, now: Instant) -> Self {
        Self {
            from: value,
            to: value,
            start: now,
        }
    }

    pub fn between(from: T, to: T, now: Instant) -> Self {
        Self {
            from,
            to,
            start: now,
        }
    }

    pub fn target(&self) -> T {
        self.to
    }

    pub fn value(&self, now: Instant) -> T {
        if self.is_done(now) {
            return self.to;
        }
        self.from.lerp(self.to, ease_out(self.progress(now)))
    }

    pub fn is_done(&self, now: Instant) -> bool {
        self.from == self.to || self.progress(now) >= 1.0
    }

    pub fn retarget(&mut self, to: T, now: Instant) {
        if to == self.to {
            return;
        }
        *self = Self::between(self.value(now), to, now);
    }

    pub fn snap(&mut self, to: T) {
        self.from = to;
        self.to = to;
    }

    pub fn steer(&mut self, to: T, now: Instant, animate: bool) -> T {
        if animate {
            self.retarget(to, now);
        } else if to != self.to {
            self.snap(to);
        }
        self.value(now)
    }

    fn progress(&self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.start);
        elapsed.as_secs_f32() / DURATION.as_secs_f32()
    }
}
