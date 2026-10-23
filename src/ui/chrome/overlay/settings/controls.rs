use crate::geometry::Rect;
use crate::theme::Rgba;
use crate::ui::chrome::{Icon, Painter};
use crate::ui::frame::{Chip, ChoiceRow, SettingControl, SettingsFrame, Stepper};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

const KNOB_INSET: f32 = 2.0;
const ON_ACCENT: Rgba = Rgba::hex(0xFFFFFF);
const DISABLED_OPACITY: f32 = 0.35;
const CHIP_STROKE: f32 = 1.0;

pub(super) fn draw_restore(painter: &mut Painter, settings: &SettingsFrame) {
    let theme = painter.theme;
    let hit = Hit::Setting(SettingControl::RestoreText);
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let line = &settings.restore;
    painter.press_fill(line.rect, hit, radius);
    painter.label(&line.label, line.label_x, line.rect, theme.text);
    draw_toggle(painter, settings.toggle, settings.restores_text);
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

pub(super) fn draw_stepper(
    painter: &mut Painter,
    stepper: &Stepper,
    [shrink, grow]: [SettingControl; 2],
) {
    let theme = painter.theme;
    let line = &stepper.line;
    painter.label(&line.label, line.label_x, line.rect, theme.text);
    let smaller = Hit::Setting(shrink);
    step_button(
        painter,
        stepper.smaller,
        smaller,
        Icon::Minimize,
        stepper.can_shrink,
    );
    let value = &stepper.value;
    let value_x = stepper.value_rect.x + (stepper.value_rect.width - value.width) / 2.0;
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
    let opacity = if enabled { 1.0 } else { DISABLED_OPACITY };
    painter.scene.push_opacity(opacity);
    painter.small_button(rect, hit, icon);
    painter.scene.pop_opacity();
}

pub(super) fn draw_choices(painter: &mut Painter, row: &ChoiceRow) {
    let line = &row.line;
    painter.label(&line.label, line.label_x, line.rect, painter.theme.text);
    for chip in &row.chips {
        draw_chip(painter, chip);
    }
    if let Some((rect, control)) = row.clear {
        painter.small_button(rect, Hit::Setting(control), Icon::Close);
    }
}

fn draw_chip(painter: &mut Painter, chip: &Chip) {
    let theme = painter.theme;
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    let color = match chip.selected {
        true => {
            painter.scene.fill(chip.rect, theme.accent, radius);
            ON_ACCENT
        }
        false => {
            let stroke = painter.metrics.px(CHIP_STROKE);
            painter
                .scene
                .border(chip.rect, theme.border, radius, stroke);
            let glow = painter.press_fill(chip.rect, Hit::Setting(chip.control), radius);
            theme.text_muted.mix(theme.text, glow)
        }
    };
    let x = chip.rect.x + (chip.rect.width - chip.label.width) / 2.0;
    painter.label(&chip.label, x, chip.rect, color);
}
