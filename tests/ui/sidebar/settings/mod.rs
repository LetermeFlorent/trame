mod choices;
mod connections;
mod folders;
mod launch;
mod message;
mod shortcuts;
mod system;
mod tabs;
mod terminals;
mod window;

use trame::geometry::Rect;
use trame::locale::{Language, Strings};
use trame::preferences::Preferences;
use trame::text::TextSystem;
use trame::ui::frame::{
    ChoiceRow, FormPage, SettingControl, SettingsFrame, SettingsRequest, SettingsTab, Stepper,
};
use trame::ui::{FrameLayout, Hit};

use crate::{frame, text_system, workspace};

pub const FORM_TABS: [SettingsTab; 4] = [
    SettingsTab::General,
    SettingsTab::Terminals,
    SettingsTab::Display,
    SettingsTab::System,
];

pub fn request_on<'a>(
    preferences: &'a Preferences,
    strings: &'static Strings,
    tab: SettingsTab,
) -> SettingsRequest<'a> {
    SettingsRequest {
        preferences,
        strings,
        folder: None,
        tab,
        hosts: None,
        shortcuts: None,
    }
}

pub fn general_request<'a>(
    preferences: &'a Preferences,
    strings: &'static Strings,
) -> SettingsRequest<'a> {
    request_on(preferences, strings, SettingsTab::General)
}

pub fn placed(text: &mut TextSystem, request: &SettingsRequest) -> FrameLayout {
    let mut layout = frame(text, &workspace());
    layout.place_settings(text, request);
    layout
}

pub fn with_settings(
    text: &mut TextSystem,
    preferences: &Preferences,
    strings: &'static Strings,
) -> FrameLayout {
    placed(text, &general_request(preferences, strings))
}

pub fn english(preferences: &Preferences) -> FrameLayout {
    with_settings(&mut text_system(), preferences, Language::English.strings())
}

pub fn on_tab(preferences: &Preferences, tab: SettingsTab) -> FrameLayout {
    let request = request_on(preferences, Language::English.strings(), tab);
    placed(&mut text_system(), &request)
}

pub fn panel(layout: &FrameLayout) -> &SettingsFrame {
    layout.settings.as_ref().expect("the settings window")
}

pub fn page(layout: &FrameLayout) -> &FormPage {
    panel(layout).form().expect("a settings form")
}

fn drawn_controls(settings: &SettingsFrame) -> Vec<(Rect, SettingControl)> {
    let page = settings.form().expect("a settings form");
    let rows = page.rows.iter().flat_map(ChoiceRow::controls);
    let toggles = page
        .toggles
        .iter()
        .filter(|toggle| toggle.enabled)
        .map(|toggle| (toggle.line.rect, toggle.control));
    let steppers = page.steppers.iter().flat_map(Stepper::targets);
    let tabs = settings.tabs.iter().map(|chip| (chip.rect, chip.control));
    [(settings.popup.close, SettingControl::Close)]
        .into_iter()
        .chain(tabs)
        .chain(steppers)
        .chain(toggles)
        .chain(rows)
        .collect()
}

#[test]
fn every_control_answers_where_it_is_drawn() {
    for tab in FORM_TABS {
        let layout = on_tab(&Preferences::default(), tab);
        let settings = panel(&layout);
        for (rect, control) in drawn_controls(settings) {
            assert!(settings.popup.rect.contains(rect.center()), "{control:?}");
            assert_eq!(
                layout.hit(rect.center(), false),
                Some(Hit::Setting(control))
            );
        }
        for line in [&settings.popup.title]
            .into_iter()
            .chain(&page(&layout).sections)
        {
            assert_eq!(
                layout.hit(line.rect.center(), false),
                Some(Hit::SettingsPanel)
            );
        }
    }
}
