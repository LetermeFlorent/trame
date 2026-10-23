use crate::geometry::Rect;
use crate::ui::frame::sidebar::settings::SettingLine;
use crate::ui::frame::sidebar::settings::column::{Column, LINE_HEIGHT, LINE_INSET};

use super::Stepper;

const VALUE_WIDTH: f32 = 44.0;
const TOGGLE_WIDTH: f32 = 28.0;
const TOGGLE_HEIGHT: f32 = 16.0;

impl Column<'_> {
    pub(super) fn toggle(&self, line: &SettingLine) -> Rect {
        let width = self.metrics.px(TOGGLE_WIDTH);
        let height = self.metrics.px(TOGGLE_HEIGHT);
        let right = line.rect.right() - self.metrics.px(LINE_INSET);
        let y = line.rect.centered_y(height);
        Rect::new(right - width, y, width, height).snapped()
    }

    pub(super) fn stepper(
        &mut self,
        label: &str,
        value: &str,
        [can_shrink, can_grow]: [bool; 2],
    ) -> Stepper {
        let line = self.line(label, LINE_HEIGHT, self.metrics.label());
        let bigger = self.end_button(&line);
        let value_width = self.metrics.px(VALUE_WIDTH);
        let value_rect = Rect::new(
            bigger.x - value_width,
            line.rect.y,
            value_width,
            line.rect.height,
        );
        let smaller = Rect {
            x: value_rect.x - bigger.width,
            ..bigger
        };
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
