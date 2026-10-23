use trame::locale::{Language, LanguageChoice};
use trame::preferences::{MAX_INTERFACE_SIZE, MIN_INTERFACE_SIZE, Preferences};
use trame::ui::metrics::{MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE};

use super::{english, panel, with_settings};
use crate::text_system;

#[test]
fn the_language_row_offers_auto_french_and_english_and_marks_the_choice() {
    for chosen in LanguageChoice::ALL {
        let layout = english(&Preferences {
            language: chosen,
            ..Preferences::default()
        });
        let settings = panel(&layout);
        let order: Vec<_> = settings
            .languages
            .iter()
            .map(|option| option.choice)
            .collect();
        assert_eq!(order, LanguageChoice::ALL.to_vec());
        for pair in settings.languages.windows(2) {
            assert!(pair[0].rect.right() <= pair[1].rect.x);
        }
        for option in &settings.languages {
            assert!(settings.language.rect.contains(option.rect.center()));
            assert_eq!(option.selected, option.choice == chosen);
        }
    }
}

#[test]
fn the_window_speaks_the_chosen_language() {
    let mut text = text_system();
    let french = Language::French.strings();
    let layout = with_settings(&mut text, &Preferences::default(), french);
    let title = &panel(&layout).title.label;
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
        restore_text: false,
        interface_size: MIN_INTERFACE_SIZE,
        ..Preferences::default()
    };
    let layout = english(&smallest);
    let settings = panel(&layout);
    assert!(!settings.restores_text);
    for stepper in [&settings.text_size, &settings.interface_size] {
        assert!(!stepper.can_shrink && stepper.can_grow);
    }
    let largest = Preferences {
        font_size: MAX_TERMINAL_FONT_SIZE,
        interface_size: MAX_INTERFACE_SIZE,
        ..Preferences::default()
    };
    let layout = english(&largest);
    for stepper in [&panel(&layout).text_size, &panel(&layout).interface_size] {
        assert!(stepper.can_shrink && !stepper.can_grow);
    }
}
