use crate::geometry::Rect;
use crate::ui::chrome::{Icon, Painter};
use crate::ui::frame::{Chip, ChoiceRow, FieldFrame, FormPage, Stepper, ToggleRow};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

use super::super::ON_ACCENT;
use super::super::editor::draw_field;
use super::super::popup::{LabelButton, draw_label_button};

const KNOB_INSET: f32 = 2.0;
const DISABLED_OPACITY: f32 = 0.35;
const VALUE_INSET_Y: f32 = 2.0;

pub(super) fn draw_toggle_row(painter: &mut Painter, row: &ToggleRow) {
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let line = &row.line;
    faded(painter, row.enabled, |painter| {
        painter.press_fill(line.rect, Hit::Setting(row.control), radius);
        painter.setting_line(line, painter.theme.text);
        draw_toggle(painter, row.switch, row.on);
    });
}

pub(super) fn draw_row(painter: &mut Painter, page: &FormPage, row: &ChoiceRow) {
    let typed = row
        .chips
        .iter()
        .find_map(|chip| page.field_for(chip.control));
    let Some(field) = typed else {
        return draw_choices(painter, row);
    };
    painter.setting_line(&row.line, painter.theme.text);
    draw_field(painter, field, true);
}

fn faded(painter: &mut Painter, enabled: bool, draw: impl FnOnce(&mut Painter)) {
    let opacity = if enabled { 1.0 } else { DISABLED_OPACITY };
    painter.scene.push_opacity(opacity);
    draw(painter);
    painter.scene.pop_opacity();
}

fn draw_toggle(painter: &mut Painter, rect: Rect, on: bool) {
    let theme = painter.theme;
    let (track, knob_color) = match on {
        true => (theme.accent, ON_ACCENT),
        false => (theme.border_strong, theme.text_muted),
    };
    painter.scene.fill(rect, track, rect.height / 2.0);
    let inset = painter.metrics.px(KNOB_INSET);
    let knob = rect.height - inset * 2.0;
    let x = match on {
        true => rect.right() - inset - knob,
        false => rect.x + inset,
    };
    let knob_rect = Rect::new(x, rect.y + inset, knob, knob);
    painter.scene.fill(knob_rect, knob_color, knob / 2.0);
}

pub(super) fn draw_stepper(painter: &mut Painter, stepper: &Stepper, field: Option<&FieldFrame>) {
    let [shrink, grow] = stepper.controls;
    let theme = painter.theme;
    let line = &stepper.line;
    painter.setting_line(line, theme.text);
    if let Some(field) = field {
        return draw_field(painter, field, true);
    }
    let smaller = Hit::Setting(shrink);
    step_button(
        painter,
        stepper.smaller,
        smaller,
        Icon::Minimize,
        stepper.can_shrink,
    );
    if let Some(control) = stepper.typed {
        let radius = painter.metrics.px(UiMetrics::RADIUS);
        let inset = painter.metrics.px(VALUE_INSET_Y);
        let rect = stepper.value_rect.inset_by(0.0, inset);
        painter.press_fill(rect, Hit::Setting(control), radius);
    }
    let value = &stepper.value;
    let value_x = stepper.value_rect.centered_x(value.width);
    painter.label(value, value_x, stepper.value_rect, theme.text_muted);
    let bigger = Hit::Setting(grow);
    step_button(
        painter,
        stepper.bigger,
        bigger,
        Icon::Plus,
        stepper.can_grow,
    );
}

fn step_button(painter: &mut Painter, rect: Rect, hit: Hit, icon: Icon, enabled: bool) {
    faded(painter, enabled, |painter| {
        painter.small_button(rect, hit, icon)
    });
}

pub(super) fn draw_choices(painter: &mut Painter, row: &ChoiceRow) {
    let line = &row.line;
    painter.setting_line(line, painter.theme.text);
    for chip in &row.chips {
        draw_chip(painter, chip);
    }
    if let Some((rect, control)) = row.clear {
        painter.small_button(rect, Hit::Setting(control), Icon::Close);
    }
}

pub(super) fn draw_header(painter: &mut Painter, row: &ChoiceRow) {
    painter.setting_line(&row.line, painter.theme.text_faint);
    for chip in &row.chips {
        draw_chip(painter, chip);
    }
}

pub(super) fn draw_chip(painter: &mut Painter, chip: &Chip) {
    let button = LabelButton {
        rect: chip.rect,
        label: &chip.label,
        hit: Hit::Setting(chip.control),
        filled: chip.selected,
        faint: chip.faint,
    };
    draw_label_button(painter, &button);
}
