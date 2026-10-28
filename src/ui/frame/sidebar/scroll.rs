use crate::geometry::Rect;
use crate::ui::metrics::UiMetrics;

pub(in crate::ui::frame) const BAR_WIDTH: f32 = 6.0;
pub(in crate::ui::frame) const BAR_GAP: f32 = 4.0;
const MIN_THUMB: f32 = 18.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollBar {
    pub track: Rect,
    pub thumb: Rect,
    pub first: usize,
    pub last_first: usize,
    pub shown: usize,
}

pub(in crate::ui::frame) struct ListWindow {
    pub(in crate::ui::frame) first: usize,
    pub(in crate::ui::frame) shown: usize,
    pub(in crate::ui::frame) count: usize,
}

impl ScrollBar {
    pub(in crate::ui::frame) fn fit(metrics: &UiMetrics, rect: Rect, window: ListWindow) -> Self {
        let ListWindow {
            first,
            shown,
            count,
        } = window;
        let width = metrics.px(BAR_WIDTH);
        let track = Rect::new(rect.right() - width, rect.y, width, rect.height);
        let share = track.height * shown as f32 / count as f32;
        let thumb_height = share.max(metrics.px(MIN_THUMB)).min(track.height).round();
        let last_first = count - shown;
        let travel = track.height - thumb_height;
        let y = track.y + travel * first as f32 / last_first as f32;
        Self {
            track,
            thumb: Rect::new(track.x, y.round(), width, thumb_height),
            first,
            last_first,
            shown,
        }
    }

    pub fn scrolled(&self, rows: isize) -> usize {
        self.first.saturating_add_signed(rows).min(self.last_first)
    }

    pub fn first_at(&self, thumb_top: f32) -> usize {
        let travel = self.track.height - self.thumb.height;
        if travel <= 0.0 {
            return self.first;
        }
        let ratio = ((thumb_top - self.track.y) / travel).clamp(0.0, 1.0);
        (ratio * self.last_first as f32).round() as usize
    }

    pub fn paged(&self, y: f32) -> usize {
        match y < self.thumb.y {
            true => self.first.saturating_sub(self.shown),
            false => (self.first + self.shown).min(self.last_first),
        }
    }
}
