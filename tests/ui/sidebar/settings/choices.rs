use trame::locale::{Language, LanguageChoice};
use trame::preferences::{MAX_INTERFACE_SIZE, MIN_INTERFACE_SIZE, Preferences};
use trame::ui::frame::{SettingControl, SettingsTab, Stepper};
use trame::ui::metrics::{MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE};
use trame::ui::{FrameLayout, Hit};

use super::{on_tab, page, panel, request_on, with_settings};
use crate::{frame, text_system, workspace};

fn sizes(layout: &FrameLayout) -> [&Stepper; 2] {
    let size = |control| page(layout).stepper(control).expect("the stepper");
    [
        SettingControl::TextSmaller,
        SettingControl::InterfaceSmaller,
    ]
    .map(size)
}

#[test]
fn the_language_row_offers_auto_french_and_english_and_marks_the_choice() {
    for chosen in LanguageChoice::ALL {
        let preferences = Preferences {
            language: chosen,
            ..Preferences::default()
        };
        let layout = on_tab(&preferences, SettingsTab::Display);
        let row = page(&layout).row(SettingControl::Language(chosen));
        let row = row.expect("the language row");
        let order: Vec<_> = row.chips.iter().map(|chip| chip.control).collect();
        assert_eq!(order, LanguageChoice::ALL.map(SettingControl::Language));
        for pair in row.chips.windows(2) {
            assert!(pair[0].rect.right() <= pair[1].rect.x);
        }
        for chip in &row.chips {
            assert!(row.line.rect.contains(chip.rect.center()));
            assert_eq!(
                chip.selected,
                chip.control == SettingControl::Language(chosen)
            );
        }
        assert!(row.clear.is_none());
    }
}

#[test]
fn the_window_speaks_the_chosen_language() {
    let mut text = text_system();
    let french = Language::French.strings();
    let layout = with_settings(&mut text, &Preferences::default(), french);
    let title = &panel(&layout).popup.title.label;
    let expected = text.layout(french.settings, layout.metrics.strong());
    assert_eq!(title.width, expected.width);
    let english = text.layout(
        Language::English.strings().settings,
        layout.metrics.strong(),
    );
    assert_ne!(title.width, english.width);
}

#[test]
fn steppers_stop_at_their_limits() {
    let smallest = Preferences {
        font_size: MIN_TERMINAL_FONT_SIZE,
        interface_size: MIN_INTERFACE_SIZE,
        ..Preferences::default()
    };
    let layout = on_tab(&smallest, SettingsTab::Display);
    for stepper in sizes(&layout) {
        assert!(!stepper.can_shrink && stepper.can_grow);
    }
    let largest = Preferences {
        font_size: MAX_TERMINAL_FONT_SIZE,
        interface_size: MAX_INTERFACE_SIZE,
        ..Preferences::default()
    };
    let layout = on_tab(&largest, SettingsTab::Display);
    for stepper in sizes(&layout) {
        assert!(stepper.can_shrink && !stepper.can_grow);
    }
}

#[test]
fn the_blur_switch_shows_only_where_the_window_can_see_through() {
    let preferences = Preferences::default();
    let request = request_on(
        &preferences,
        Language::English.strings(),
        SettingsTab::Display,
    );
    for backdrop in [false, true] {
        let mut text = text_system();
        let mut layout = frame(&mut text, &workspace());
        layout.metrics.backdrop = backdrop;
        layout.place_settings(&mut text, &request);
        let blur = page(&layout).toggle(SettingControl::Blur);
        assert_eq!(blur.is_some(), backdrop);
        if let Some(blur) = blur {
            assert!(!blur.on && blur.enabled);
            let hit = layout.hit(blur.line.rect.center(), false);
            assert_eq!(hit, Some(Hit::Setting(SettingControl::Blur)));
        }
    }
}
