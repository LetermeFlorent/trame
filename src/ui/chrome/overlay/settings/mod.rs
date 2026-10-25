mod connections;
mod controls;

use crate::render::Scene;
use crate::text::TextSystem;
use crate::ui::chrome::{ChromeInput, Painter};
use crate::ui::frame::{GeneralPage, InstancesPage, SettingControl, SettingsPage};
use crate::ui::hit::Hit;
use connections::draw_connections;
use controls::{draw_chip, draw_choices, draw_command, draw_stepper, draw_toggle_row};

use super::popup::draw_popup;

pub fn draw_settings(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let Some(settings) = &input.frame.settings else {
        return;
    };
    let mut painter = Painter::new(scene, text, input);
    let close = Hit::Setting(SettingControl::Close);
    draw_popup(&mut painter, &settings.popup, close);
    for tab in &settings.tabs {
        draw_chip(&mut painter, tab);
    }
    match &settings.page {
        SettingsPage::General(page) => draw_general(&mut painter, page),
        SettingsPage::Instances(page) => draw_instances(&mut painter, page),
        SettingsPage::Connections(page) => draw_connections(&mut painter, page),
    }
}

fn draw_instances(painter: &mut Painter, page: &InstancesPage) {
    painter.setting_line(&page.section, painter.theme.text_faint);
    for toggle in &page.toggles {
        draw_toggle_row(painter, toggle);
    }
}

fn draw_general(painter: &mut Painter, page: &GeneralPage) {
    let faint = painter.theme.text_faint;
    for section in &page.sections {
        painter.setting_line(section, faint);
    }
    for toggle in page.all_toggles() {
        draw_toggle_row(painter, toggle);
    }
    draw_command(painter, page);
    for (stepper, controls) in page.steppers() {
        draw_stepper(painter, stepper, controls);
    }
    for row in [
        &page.language,
        &page.panel_side,
        &page.density,
        &page.folder,
    ] {
        draw_choices(painter, row);
    }
}
