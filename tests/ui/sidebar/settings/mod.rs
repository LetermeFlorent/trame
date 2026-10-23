mod choices;
mod connections;
mod folders;

use trame::geometry::Point;
use trame::locale::{Language, Strings};
use trame::preferences::Preferences;
use trame::text::TextSystem;
use trame::ui::frame::{ChoiceRow, SettingControl, SettingsFrame};
use trame::ui::{FrameLayout, Hit};

use super::footer::closed_sidebar;
use crate::{frame, text_system, workspace};

pub fn with_settings(
    text: &mut TextSystem,
    preferences: &Preferences,
    strings: &Strings,
) -> FrameLayout {
    let mut layout = frame(text, &workspace());
    layout.place_settings(text, preferences, strings);
    layout
}

pub fn english(preferences: &Preferences) -> FrameLayout {
    with_settings(&mut text_system(), preferences, Language::English.strings())
}

pub fn panel(layout: &FrameLayout) -> &SettingsFrame {
    layout.settings.as_ref().expect("the settings window")
}

#[test]
fn the_window_opens_in_the_middle_over_everything_below_the_title_bar() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    assert_eq!(settings.backdrop.y, layout.title_bar.rect.bottom());
    assert_eq!(settings.backdrop.bottom(), layout.bounds.bottom());
    assert_eq!(settings.backdrop.width, layout.bounds.width);
    let (panel, backdrop) = (settings.rect.center(), settings.backdrop.center());
    assert!((panel.x - backdrop.x).abs() <= 1.0);
    assert!((panel.y - backdrop.y).abs() <= 1.0);
}

#[test]
fn every_control_answers_where_it_is_drawn() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    let rows = [&settings.language, &settings.folder]
        .into_iter()
        .chain(&settings.hosts)
        .flat_map(ChoiceRow::controls);
    let controls = [
        (settings.close, SettingControl::Close),
        (settings.restore.rect, SettingControl::RestoreText),
        (settings.text_size.smaller, SettingControl::TextSmaller),
        (settings.text_size.bigger, SettingControl::TextBigger),
        (
            settings.interface_size.smaller,
            SettingControl::InterfaceSmaller,
        ),
        (
            settings.interface_size.bigger,
            SettingControl::InterfaceBigger,
        ),
    ];
    for (rect, control) in controls.into_iter().chain(rows) {
        assert!(settings.rect.contains(rect.center()), "{control:?}");
        assert_eq!(
            layout.hit(rect.center(), false),
            Some(Hit::Setting(control))
        );
    }
    for line in [&settings.title].into_iter().chain(&settings.sections) {
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
    let content = Point::new(settings.rect.right() + 10.0, settings.rect.center().y);
    for point in [row.center(), content, layout.footer.settings.center()] {
        assert_eq!(layout.hit(point, false), Some(Hit::Backdrop));
    }
    let controls = layout.title_bar.controls.expect("window buttons");
    assert_eq!(layout.hit(controls.close.center(), false), Some(Hit::Close));
}

#[test]
fn a_collapsed_sidebar_changes_nothing_to_the_window() {
    let mut text = text_system();
    let mut layout = frame(&mut text, &closed_sidebar());
    layout.place_settings(
        &mut text,
        &Preferences::default(),
        Language::English.strings(),
    );
    let open = english(&Preferences::default());
    assert_eq!(panel(&layout).rect, panel(&open).rect);
}
