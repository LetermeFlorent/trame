use crate::geometry::Rect;
use crate::ui::frame::fit_label;
use crate::ui::metrics::UiMetrics;

use super::column::{BUTTON_GAP, Column, LINE_HEIGHT, middle};
use super::{Chip, ChoiceRow, SettingControl, SettingLine};

const CHOICE_PAD: f32 = 8.0;

pub(super) type Choice<'a> = (&'a str, SettingControl, bool);

impl Column<'_> {
    pub(super) fn chip_room(&self) -> f32 {
        self.inner.width / 2.0 - 2.0 * self.metrics.px(CHOICE_PAD)
    }

    fn chip_on(&mut self, line: &SettingLine, choice: Choice) -> Chip {
        let pad = self.metrics.px(CHOICE_PAD);
        self.padded_chip(line, choice, pad)
    }

    fn padded_chip(
        &mut self,
        line: &SettingLine,
        (text, control, selected): Choice,
        pad: f32,
    ) -> Chip {
        let height = self.metrics.px(UiMetrics::BUTTON);
        let label = fit_label(self.text, text, self.metrics.label(), self.chip_room());
        let width = (label.width + 2.0 * pad).ceil();
        Chip {
            rect: Rect::new(line.rect.x, middle(line, height), width, height),
            label,
            control,
            selected,
            faint: false,
        }
    }

    pub(super) fn choice_row(
        &mut self,
        label: &str,
        choices: &[Choice],
        clear: Option<SettingControl>,
    ) -> ChoiceRow {
        let line = self.line(label, LINE_HEIGHT, self.metrics.label());
        self.choices_on(line, choices, clear)
    }

    pub(super) fn choices_on(
        &mut self,
        line: SettingLine,
        choices: &[Choice],
        clear: Option<SettingControl>,
    ) -> ChoiceRow {
        let clear = clear.map(|control| (self.end_button(&line), control));
        let gap = self.metrics.px(BUTTON_GAP);
        let mut right = clear.map_or(line.rect.right(), |(rect, _)| rect.x - gap);
        let mut chips: Vec<Chip> = choices
            .iter()
            .rev()
            .map(|&choice| {
                let mut chip = self.chip_on(&line, choice);
                chip.rect.x = right - chip.rect.width;
                right = chip.rect.x - gap;
                chip
            })
            .collect();
        chips.reverse();
        ChoiceRow { line, chips, clear }
    }

    pub(super) fn tab_row<const N: usize>(&mut self, choices: [Choice; N]) -> [Chip; N] {
        let line = self.line("", LINE_HEIGHT, self.metrics.label());
        let pad = self.metrics.px(CHOICE_PAD);
        let tabs = self.tabs_on(&line, choices, pad);
        let over = tabs
            .last()
            .map_or(0.0, |tab| tab.rect.right() - line.rect.right());
        if over <= 0.0 {
            return tabs;
        }
        let squeeze = (over / (2 * N) as f32).ceil();
        self.tabs_on(&line, choices, (pad - squeeze).max(0.0))
    }

    fn tabs_on<const N: usize>(
        &mut self,
        line: &SettingLine,
        choices: [Choice; N],
        pad: f32,
    ) -> [Chip; N] {
        let gap = self.metrics.px(BUTTON_GAP);
        let mut left = line.rect.x;
        choices.map(|choice| {
            let mut chip = self.padded_chip(line, choice, pad);
            chip.rect.x = left;
            left = chip.rect.right() + gap;
            chip
        })
    }
}
