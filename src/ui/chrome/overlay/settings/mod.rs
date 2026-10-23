mod controls;

use crate::render::Scene;
use crate::text::TextSystem;
use crate::ui::chrome::{ChromeInput, Icon, Painter};
use crate::ui::frame::{SettingControl, SettingsFrame};
use crate::ui::hit::Hit;
use controls::{draw_languages, draw_restore, draw_stepper};

const PANEL_BLUR: f32 = 8.0;
const BACKDROP_ALPHA: f32 = 0.35;

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
    draw_languages(&mut painter, settings);
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
