mod connections;
mod controls;
mod shortcuts;

use crate::render::Scene;
use crate::text::TextSystem;
use crate::ui::chrome::{ChromeInput, Painter};
use crate::ui::frame::{FormPage, InstancesPage, SettingControl, SettingsPage};
use crate::ui::hit::Hit;
use connections::draw_connections;
use controls::{draw_chip, draw_row, draw_stepper, draw_toggle_row};
use shortcuts::draw_shortcuts;

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
    painter.scene.push_clip(settings.popup.rect);
    match &settings.page {
        SettingsPage::Form(page) => draw_form(&mut painter, page),
        SettingsPage::Instances(page) => draw_instances(&mut painter, page),
        SettingsPage::Connections(page) => draw_connections(&mut painter, page),
        SettingsPage::Shortcuts(page) => draw_shortcuts(&mut painter, page),
    }
    painter.scene.pop_clip();
}

fn draw_instances(painter: &mut Painter, page: &InstancesPage) {
    painter.setting_line(&page.section, painter.theme.text_faint);
    for toggle in &page.toggles {
        draw_toggle_row(painter, toggle);
    }
}

fn draw_form(painter: &mut Painter, page: &FormPage) {
    let faint = painter.theme.text_faint;
    for section in &page.sections {
        painter.setting_line(section, faint);
    }
    for toggle in &page.toggles {
        draw_toggle_row(painter, toggle);
    }
    for stepper in &page.steppers {
        let field = stepper.typed.and_then(|control| page.field_for(control));
        draw_stepper(painter, stepper, field);
    }
    for row in &page.rows {
        draw_row(painter, page, row);
    }
}
