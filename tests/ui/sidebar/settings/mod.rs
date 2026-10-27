mod choices;
mod connections;
mod folders;
mod launch;
mod message;
mod shortcuts;
mod tabs;

use trame::geometry::{Point, Rect};
use trame::locale::{Language, Strings};
use trame::preferences::Preferences;
use trame::text::TextSystem;
use trame::ui::frame::{
    ChoiceRow, GeneralPage, SettingControl, SettingsFrame, SettingsRequest, SettingsTab, ToggleRow,
};
use trame::ui::{FrameLayout, Hit};

use super::footer::closed_sidebar;
use crate::{frame, text_system, workspace};

pub fn general_request<'a>(
    preferences: &'a Preferences,
    strings: &'static Strings,
) -> SettingsRequest<'a> {
    SettingsRequest {
        preferences,
        strings,
        folder: None,
        tab: SettingsTab::General,
        hosts: None,
        shortcuts: None,
    }
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

pub fn panel(layout: &FrameLayout) -> &SettingsFrame {
    layout.settings.as_ref().expect("the settings window")
}

pub fn page(layout: &FrameLayout) -> &GeneralPage {
    panel(layout).general().expect("the general page")
}

#[test]
fn the_window_opens_in_the_middle_over_everything_below_the_title_bar() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    assert_eq!(settings.popup.backdrop.y, layout.title_bar.rect.bottom());
    assert_eq!(settings.popup.backdrop.bottom(), layout.bounds.bottom());
    assert_eq!(settings.popup.backdrop.width, layout.bounds.width);
    let (panel, backdrop) = (
        settings.popup.rect.center(),
        settings.popup.backdrop.center(),
    );
    assert!((panel.x - backdrop.x).abs() <= 1.0);
    assert!((panel.y - backdrop.y).abs() <= 1.0);
}

fn drawn_controls(settings: &SettingsFrame) -> Vec<(Rect, SettingControl)> {
    let page = settings.general().expect("the general page");
    let rows = [&page.command, &page.language, &page.folder]
        .into_iter()
        .flat_map(ChoiceRow::controls);
    let toggles = page
        .all_toggles()
        .map(|toggle: &ToggleRow| (toggle.line.rect, toggle.control));
    let steppers = page
        .steppers()
        .into_iter()
        .flat_map(|(stepper, [shrink, grow])| [(stepper.smaller, shrink), (stepper.bigger, grow)]);
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
    let layout = english(&Preferences::default());
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

#[test]
fn a_click_beside_the_window_reaches_nothing_but_the_title_bar_still_works() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    let row = layout.sidebar.as_ref().expect("the sidebar").rows[0].rect;
    let content = Point::new(
        settings.popup.rect.right() + 10.0,
        settings.popup.rect.center().y,
    );
    for point in [row.center(), content, layout.footer.settings.rect.center()] {
        assert_eq!(layout.hit(point, false), Some(Hit::Backdrop));
    }
    if let Some(controls) = layout.title_bar.controls {
        assert_eq!(layout.hit(controls.close.center(), false), Some(Hit::Close));
    }
}

#[test]
fn a_collapsed_sidebar_changes_nothing_to_the_window() {
    let mut text = text_system();
    let mut layout = frame(&mut text, &closed_sidebar());
    let preferences = Preferences::default();
    let request = general_request(&preferences, Language::English.strings());
    layout.place_settings(&mut text, &request);
    let open = english(&Preferences::default());
    assert_eq!(panel(&layout).popup.rect, panel(&open).popup.rect);
}
