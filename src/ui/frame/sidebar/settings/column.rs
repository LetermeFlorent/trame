use crate::geometry::Rect;
use crate::text::{TextStyle, TextSystem};
use crate::ui::metrics::UiMetrics;

use super::{SettingLine, Stepper};

pub(super) const CAPTION_HEIGHT: f32 = 22.0;
pub(super) const LINE_HEIGHT: f32 = 28.0;

const LINE_INSET: f32 = 6.0;
const VALUE_WIDTH: f32 = 44.0;
const TOGGLE_WIDTH: f32 = 28.0;
const TOGGLE_HEIGHT: f32 = 16.0;

pub(super) struct Column<'a> {
    pub(super) text: &'a mut TextSystem,
    pub(super) metrics: UiMetrics,
    pub(super) inner: Rect,
    pub(super) y: f32,
}

impl Column<'_> {
    pub(super) fn line(&mut self, label: &str, height: f32, style: TextStyle) -> SettingLine {
        let rect = Rect::new(
            self.inner.x,
            self.y,
            self.inner.width,
            self.metrics.px(height),
        );
        self.y = rect.bottom();
        SettingLine {
            rect,
            label: self.text.layout(label, style),
            label_x: rect.x + self.metrics.px(LINE_INSET),
        }
    }

    pub(super) fn toggle(&self, line: &SettingLine) -> Rect {
        let width = self.metrics.px(TOGGLE_WIDTH);
        let height = self.metrics.px(TOGGLE_HEIGHT);
        let right = line.rect.right() - self.metrics.px(LINE_INSET);
        let y = line.rect.y + (line.rect.height - height) / 2.0;
        Rect::new(right - width, y, width, height).snapped()
    }

    pub(super) fn stepper(
        &mut self,
        label: &str,
        value: &str,
        [can_shrink, can_grow]: [bool; 2],
    ) -> Stepper {
        let line = self.line(label, LINE_HEIGHT, self.metrics.label());
        let button = self.metrics.px(UiMetrics::BUTTON);
        let value_width = self.metrics.px(VALUE_WIDTH);
        let y = (line.rect.y + (line.rect.height - button) / 2.0).round();
        let bigger = Rect::new(line.rect.right() - button, y, button, button);
        let value_rect = Rect::new(
            bigger.x - value_width,
            line.rect.y,
            value_width,
            line.rect.height,
        );
        let smaller = Rect::new(value_rect.x - button, y, button, button);
        Stepper {
            value: self.text.layout(value, self.metrics.label()),
            line,
            smaller,
            bigger,
            value_rect,
            can_shrink,
            can_grow,
        }
    }
}
