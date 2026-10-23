use crate::geometry::Rect;
use crate::text::{TextStyle, TextSystem};
use crate::ui::frame::fit_label;
use crate::ui::metrics::UiMetrics;

use super::{Chip, ChoiceRow, SettingControl, SettingLine, Stepper};

pub(super) const CAPTION_HEIGHT: f32 = 22.0;
pub(super) const LINE_HEIGHT: f32 = 28.0;
pub(super) const SECTION_GAP: f32 = 8.0;

const LINE_INSET: f32 = 6.0;
const VALUE_WIDTH: f32 = 44.0;
const TOGGLE_WIDTH: f32 = 28.0;
const TOGGLE_HEIGHT: f32 = 16.0;
const CHOICE_PAD: f32 = 8.0;
const CHOICE_GAP: f32 = 2.0;

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
        let inset = self.metrics.px(LINE_INSET);
        let widest = rect.width - self.metrics.px(UiMetrics::BUTTON) - 2.0 * inset;
        SettingLine {
            rect,
            label: fit_label(self.text, label, style, widest),
            label_x: rect.x + inset,
        }
    }

    pub(super) fn caption(&mut self, label: &str) -> SettingLine {
        self.line(label, CAPTION_HEIGHT, self.metrics.caption())
    }

    pub(super) fn section(&mut self, label: &str) -> SettingLine {
        self.y += self.metrics.px(SECTION_GAP);
        self.caption(label)
    }

    pub(super) fn toggle(&self, line: &SettingLine) -> Rect {
        let width = self.metrics.px(TOGGLE_WIDTH);
        let height = self.metrics.px(TOGGLE_HEIGHT);
        let right = line.rect.right() - self.metrics.px(LINE_INSET);
        let y = line.rect.y + (line.rect.height - height) / 2.0;
        Rect::new(right - width, y, width, height).snapped()
    }

    pub(super) fn end_button(&self, line: &SettingLine) -> Rect {
        let button = self.metrics.px(UiMetrics::BUTTON);
        let y = middle(line, button);
        Rect::new(line.rect.right() - button, y, button, button)
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

    pub(super) fn choice_row(
        &mut self,
        label: &str,
        choices: &[(&str, SettingControl, bool)],
        clear: Option<SettingControl>,
    ) -> ChoiceRow {
        let line = self.line(label, LINE_HEIGHT, self.metrics.label());
        let clear = clear.map(|control| (self.end_button(&line), control));
        let height = self.metrics.px(UiMetrics::BUTTON);
        let pad = self.metrics.px(CHOICE_PAD);
        let gap = self.metrics.px(CHOICE_GAP);
        let widest = line.rect.width / 2.0 - 2.0 * pad;
        let y = middle(&line, height);
        let mut right = clear.map_or(line.rect.right(), |(rect, _)| rect.x - gap);
        let mut chips: Vec<Chip> = choices
            .iter()
            .rev()
            .map(|&(text, control, selected)| {
                let label = fit_label(self.text, text, self.metrics.label(), widest);
                let width = (label.width + 2.0 * pad).ceil();
                let rect = Rect::new(right - width, y, width, height);
                right = rect.x - gap;
                Chip {
                    rect,
                    label,
                    control,
                    selected,
                }
            })
            .collect();
        chips.reverse();
        ChoiceRow { line, chips, clear }
    }
}

fn middle(line: &SettingLine, height: f32) -> f32 {
    (line.rect.y + (line.rect.height - height) / 2.0).round()
}
