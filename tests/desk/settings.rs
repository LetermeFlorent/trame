use trame::desk::Effect;
use trame::display::{Density, PanelSide};
use trame::locale::LanguageChoice;
use trame::preferences::MAX_HISTORY_LINES;
use trame::session::SessionId;
use trame::workspace::Numbering;

use crate::support::desk;

#[test]
fn equal_preferences_change_nothing() {
    let mut desk = desk();
    let same = desk.preferences.clone();
    assert!(desk.store_preferences(same.clone()).is_empty());
    assert!(desk.follow_preferences(&same).is_empty());
}

#[test]
fn layout_changes_ask_for_a_rescale() {
    let mut desk = desk();
    let bigger = desk.preferences.step_font_size(1);
    let effects = desk.store_preferences(bigger);
    assert_eq!(effects.as_slice(), [Effect::Rescale]);
    let right = desk
        .preferences
        .with(|kept| kept.panel_side = PanelSide::Right);
    let effects = desk.store_preferences(right);
    assert_eq!(effects.as_slice(), [Effect::Rescale]);
    let spacious = desk
        .preferences
        .with(|kept| kept.density = Density::Spacious);
    let effects = desk.store_preferences(spacious);
    assert_eq!(effects.as_slice(), [Effect::Rescale]);
}

#[test]
fn other_changes_are_kept_and_redrawn() {
    let mut desk = desk();
    let wanted = !desk.preferences.confirm_paste;
    let next = desk.preferences.with(|kept| kept.confirm_paste = wanted);
    let effects = desk.follow_preferences(&next);
    assert_eq!(effects.as_slice(), [Effect::Changed]);
    assert_eq!(desk.preferences.confirm_paste, wanted);
}

#[test]
fn a_new_language_changes_the_strings() {
    let mut desk = desk();
    for language in [LanguageChoice::French, LanguageChoice::English] {
        let next = desk.preferences.with(|kept| kept.language = language);
        let _ = desk.store_preferences(next);
        assert!(std::ptr::eq(desk.strings, language.resolve().strings()));
    }
}

#[test]
fn numbering_follows_the_preferences() {
    let mut desk = desk();
    desk.workspace.add_instance(None, SessionId(1));
    let instance = &desk.workspace.instances[0];
    assert_ne!(desk.workspace.shown_emblem(instance), instance.emblem);
    let numbering = Numbering {
        shown: false,
        ..Numbering::default()
    };
    let next = desk.preferences.with(|kept| kept.numbering = numbering);
    let _ = desk.store_preferences(next);
    let instance = &desk.workspace.instances[0];
    assert_eq!(desk.workspace.shown_emblem(instance), instance.emblem);
}

#[test]
fn stored_preferences_are_sanitized() {
    let mut desk = desk();
    let next = desk
        .preferences
        .with(|kept| kept.history_lines = usize::MAX);
    let _ = desk.store_preferences(next.clone());
    assert_eq!(desk.preferences.history_lines, MAX_HISTORY_LINES);
    assert_eq!(desk.preferences, next.sanitized());
}
