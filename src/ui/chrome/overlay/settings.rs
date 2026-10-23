use crate::geometry::Rect;
use crate::render::Scene;
use crate::text::TextSystem;
use crate::theme::Rgba;
use crate::ui::chrome::{ChromeInput, Icon, Painter};
use crate::ui::frame::{SettingControl, SettingsFrame, Stepper};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

const PANEL_BLUR: f32 = 8.0;
const KNOB_INSET: f32 = 2.0;
const KNOB_ON: Rgba = Rgba::hex(0xFFFFFF);
const DISABLED_OPACITY: f32 = 0.35;

pub fn draw_settings(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let Some(settings) = &input.frame.settings else {
        return;
    };
    let mut painter = Painter::new(scene, text, input);
    draw_panel(&mut painter, settings);
    draw_restore(&mut painter, settings);
    let text_steps = [SettingControl::TextSmaller, SettingControl::TextBigger];
    draw_stepper(&mut painter, &settings.text_size, text_steps);
    let interface_steps = [
        SettingControl::InterfaceSmaller,
        SettingControl::InterfaceBigger,
    ];
    draw_stepper(&mut painter, &settings.interface_size, interface_steps);
}

fn draw_panel(painter: &mut Painter, settings: &SettingsFrame) {
    let theme = painter.theme;
    painter.panel(settings.rect, PANEL_BLUR);
    let caption = &settings.caption;
    painter.label(
        &caption.label,
        caption.label_x,
        caption.rect,
        theme.text_faint,
    );
}

fn draw_restore(painter: &mut Painter, settings: &SettingsFrame) {
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
        true => (theme.accent, KNOB_ON),
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

fn draw_stepper(painter: &mut Painter, stepper: &Stepper, [shrink, grow]: [SettingControl; 2]) {
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
