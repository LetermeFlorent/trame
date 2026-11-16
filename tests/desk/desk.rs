use trame::theme::Appearance;

use crate::support::desk;

#[test]
fn a_new_desk_starts_empty_in_the_chosen_language_and_appearance() {
    let desk = desk();
    assert!(desk.workspace.is_empty());
    assert!(std::ptr::eq(
        desk.strings,
        desk.preferences.language.resolve().strings()
    ));
    assert_eq!(desk.theme.appearance, Appearance::Dark);
    assert!(!desk.saver.owns());
    assert!(!desk.quitting());
}
