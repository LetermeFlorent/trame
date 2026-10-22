mod fades;
mod tween;

pub use fades::{Fades, surfaces};
pub use tween::{DURATION, Lerp, Tween, ease_out};

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::geometry::Rect;
use crate::session::SessionId;
use crate::ui::hit::Hit;
use crate::workspace::PaneId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionKey {
    Pane(PaneId),
    Tab(PaneId, SessionId),
}

#[derive(Debug)]
pub struct Motion {
    now: Instant,
    animate: bool,
    sidebar: Option<Tween<f32>>,
    rects: HashMap<MotionKey, Tween<Rect>>,
    seen: HashSet<MotionKey>,
    fades: Fades,
    scrolls: HashMap<SessionId, Tween<f32>>,
}

impl Default for Motion {
    fn default() -> Self {
        Self::new(Instant::now())
    }
}

impl Motion {
    pub fn new(now: Instant) -> Self {
        Self {
            now,
            animate: false,
            sidebar: None,
            rects: HashMap::new(),
            seen: HashSet::new(),
            fades: Fades::default(),
            scrolls: HashMap::new(),
        }
    }

    pub fn now(&self) -> Instant {
        self.now
    }

    pub fn tick(&mut self, now: Instant) {
        self.now = now.max(self.now);
        self.scrolls.retain(|_, tween| !tween.is_done(now));
    }

    pub fn animate_next(&mut self) {
        self.animate = true;
    }

    pub fn is_busy(&self) -> bool {
        let now = self.now;
        self.sidebar.is_some_and(|tween| !tween.is_done(now))
            || self.rects.values().any(|tween| !tween.is_done(now))
            || self.fades.is_busy(now)
            || self.scrolls.values().any(|tween| !tween.is_done(now))
    }

    pub fn sidebar(&mut self, open: bool) -> f32 {
        let target = if open { 1.0 } else { 0.0 };
        let now = self.now;
        let animate = self.animate;
        let tween = self
            .sidebar
            .get_or_insert_with(|| Tween::settled(target, now));
        match animate {
            true => tween.retarget(target, now),
            false if tween.target() != target => tween.snap(target),
            false => {}
        }
        tween.value(now)
    }

    pub fn knows(&self, key: MotionKey) -> bool {
        self.rects.contains_key(&key)
    }

    pub fn previous(&self, key: MotionKey) -> Option<Rect> {
        self.rects.get(&key).map(Tween::target)
    }

    pub fn place(&mut self, key: MotionKey, target: Rect, entry: Option<Rect>) -> Rect {
        self.seen.insert(key);
        let now = self.now;
        let animate = self.animate;
        let tween = self.rects.entry(key).or_insert_with(|| match entry {
            Some(start) if animate => Tween::between(start, target, now),
            _ => Tween::settled(target, now),
        });
        match animate {
            true => tween.retarget(target, now),
            false if tween.target() != target => tween.snap(target),
            false => {}
        }
        tween.value(now)
    }

    pub fn finish_layout(&mut self) {
        let seen = std::mem::take(&mut self.seen);
        self.rects.retain(|key, _| seen.contains(key));
        self.animate = false;
    }

    pub fn hover(&mut self, hit: Option<Hit>) {
        self.fades.hover(hit, self.now);
    }

    pub fn glow(&self, hit: Hit) -> f32 {
        self.fades.glow(hit, self.now)
    }

    pub fn scroll_by(&mut self, id: SessionId, lines: f32, limit: f32) {
        let current = self.scroll_offset(id);
        let start = (current + lines).clamp(-limit, limit);
        self.scrolls
            .insert(id, Tween::between(start, 0.0, self.now));
    }

    pub fn scroll_offset(&self, id: SessionId) -> f32 {
        self.scrolls
            .get(&id)
            .map_or(0.0, |tween| tween.value(self.now))
    }

    pub fn stop_scroll(&mut self, id: SessionId) {
        self.scrolls.remove(&id);
    }
}
