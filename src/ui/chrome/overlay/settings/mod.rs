mod controls;

use crate::render::Scene;
use crate::text::TextSystem;
use crate::ui::chrome::{ChromeInput, Icon, Painter};
use crate::ui::frame::{SettingControl, SettingsFrame};
use crate::ui::hit::Hit;
use controls::{draw_choices, draw_command, draw_stepper, draw_toggle_row};

const PANEL_BLUR: f32 = 8.0;
const BACKDROP_ALPHA: f32 = 0.35;

pub fn draw_settings(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let Some(settings) = &input.frame.settings else {
        return;
    };
    let mut painter = Painter::new(scene, text, input);
    draw_panel(&mut painter, settings);
    for toggle in &settings.toggles {
        draw_toggle_row(&mut painter, toggle);
    }
    draw_command(&mut painter, settings);
    draw_steppers(&mut painter, settings);
    let rows = [&settings.language, &settings.folder]
        .into_iter()
        .chain(&settings.hosts);
    for row in rows {
        draw_choices(&mut painter, row);
    }
    if let Some(hint) = &settings.hosts_hint {
        let muted = painter.theme.text_muted;
        painter.label(&hint.label, hint.label_x, hint.rect, muted);
    }
}

fn draw_steppers(painter: &mut Painter, settings: &SettingsFrame) {
    let steppers = [
        (
            &settings.instance_tabs,
            [
                SettingControl::InstanceTabsFewer,
                SettingControl::InstanceTabsMore,
            ],
        ),
        (
            &settings.text_size,
            [SettingControl::TextSmaller, SettingControl::TextBigger],
        ),
        (
            &settings.interface_size,
            [
                SettingControl::InterfaceSmaller,
                SettingControl::InterfaceBigger,
            ],
        ),
    ];
    for (stepper, controls) in steppers {
        draw_stepper(painter, stepper, controls);
    }
}

fn draw_panel(painter: &mut Painter, settings: &SettingsFrame) {
    let theme = painter.theme;
    let shade = theme.shadow.with_alpha(BACKDROP_ALPHA);
    painter.scene.fill(settings.backdrop, shade, 0.0);
    painter.panel(settings.rect, PANEL_BLUR);
    let title = &settings.title;
    painter.label(&title.label, title.label_x, title.rect, theme.text);
    let close = Hit::Setting(SettingControl::Close);
    painter.button(settings.close, close, Icon::Close);
    for section in &settings.sections {
        painter.label(
            &section.label,
            section.label_x,
            section.rect,
            theme.text_faint,
        );
    }
}
