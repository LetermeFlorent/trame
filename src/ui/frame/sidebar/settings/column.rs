use crate::geometry::Rect;
use crate::text::{TextStyle, TextSystem};
use crate::ui::frame::fit_label;
use crate::ui::metrics::UiMetrics;

use super::SettingLine;

pub(in crate::ui::frame) const CAPTION_HEIGHT: f32 = 22.0;
pub(in crate::ui::frame) const LINE_HEIGHT: f32 = 28.0;
pub(in crate::ui::frame) const SECTION_GAP: f32 = 8.0;
pub(in crate::ui::frame) const LINE_INSET: f32 = 6.0;
pub(super) const BUTTON_GAP: f32 = 2.0;

pub(in crate::ui::frame) struct Column<'a> {
    pub(in crate::ui::frame) text: &'a mut TextSystem,
    pub(in crate::ui::frame) metrics: UiMetrics,
    pub(in crate::ui::frame) inner: Rect,
    pub(in crate::ui::frame) y: f32,
}

impl Column<'_> {
    pub(in crate::ui::frame) fn line(
        &mut self,
        label: &str,
        height: f32,
        style: TextStyle,
    ) -> SettingLine {
        self.line_beside(label, height, style, 1.0)
    }

    pub(super) fn line_beside(
        &mut self,
        label: &str,
        height: f32,
        style: TextStyle,
        buttons: f32,
    ) -> SettingLine {
        let rect = Rect::new(
            self.inner.x,
            self.y,
            self.inner.width,
            self.metrics.px(height),
        );
        self.y = rect.bottom();
        let inset = self.metrics.px(LINE_INSET);
        let widest = rect.width - buttons * self.metrics.px(UiMetrics::BUTTON) - 2.0 * inset;
        SettingLine {
            rect,
            label: fit_label(self.text, label, style, widest),
            label_x: rect.x + inset,
        }
    }

    pub(in crate::ui::frame) fn caption(&mut self, label: &str) -> SettingLine {
        self.line(label, CAPTION_HEIGHT, self.metrics.caption())
    }

    pub(in crate::ui::frame) fn section(&mut self, label: &str) -> SettingLine {
        self.y += self.metrics.px(SECTION_GAP);
        self.caption(label)
    }

    pub(super) fn end_button(&self, line: &SettingLine) -> Rect {
        let button = self.metrics.px(UiMetrics::BUTTON);
        let y = middle(line, button);
        Rect::new(line.rect.right() - button, y, button, button)
    }
}

pub(super) fn middle(line: &SettingLine, height: f32) -> f32 {
    line.rect.centered_y(height).round()
}
