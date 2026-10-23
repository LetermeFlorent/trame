use crate::geometry::Rect;
use crate::text::{TextStyle, TextSystem};
use crate::ui::metrics::UiMetrics;

const CLOSE_MARGIN: f32 = 5.0;
const LABEL_GAP: f32 = 6.0;

pub(super) struct TabSizing {
    pub(super) style: TextStyle,
    pub(super) padding: f32,
    pub(super) fixed: f32,
    pub(super) minimum: f32,
    maximum: f32,
    close: f32,
    close_margin: f32,
}

impl TabSizing {
    pub(super) fn new(metrics: &UiMetrics) -> Self {
        let padding = metrics.px(UiMetrics::TAB_PADDING);
        let close = metrics.px(UiMetrics::TAB_CLOSE);
        let close_margin = metrics.px(CLOSE_MARGIN);
        Self {
            style: metrics.label(),
            padding,
            fixed: padding + metrics.px(LABEL_GAP) + close + close_margin,
            minimum: metrics.px(UiMetrics::TAB_MIN),
            maximum: metrics.px(UiMetrics::TAB_MAX),
            close,
            close_margin,
        }
    }

    pub(super) fn natural(&self, text: &mut TextSystem, title: &str) -> f32 {
        (text.layout(title, self.style).width + self.fixed)
            .ceil()
            .clamp(self.minimum, self.maximum)
    }

    pub(super) fn close_rect(&self, tab: Rect) -> Rect {
        Rect::new(
            tab.right() - self.close_margin - self.close,
            tab.centered_y(self.close).round(),
            self.close,
            self.close,
        )
    }
}

pub(super) fn fit_widths(natural: &[f32], available: f32, minimum: f32) -> Vec<f32> {
    let total: f32 = natural.iter().sum();
    if total <= available || total <= 0.0 {
        return natural.to_vec();
    }
    let scale = available / total;
    natural
        .iter()
        .map(|width| (width * scale).floor().max(minimum))
        .collect()
}

pub(super) fn scroll_offset(widths: &[f32], active: usize, available: f32) -> f32 {
    let end: f32 = widths.iter().take(active + 1).sum();
    (end - available).max(0.0)
}
