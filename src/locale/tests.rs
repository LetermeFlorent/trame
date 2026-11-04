mod slots;
mod texts;

use super::*;

#[test]
fn a_french_tag_reads_as_french_whatever_its_region() {
    for tag in ["fr", "fr-FR", "fr_CA.UTF-8", "FR_be", " fr@euro"] {
        assert_eq!(language_of_tag(tag), Language::French, "{tag}");
    }
}

#[test]
fn any_other_or_empty_tag_reads_as_english() {
    for tag in ["", "C", "POSIX", "en_US.UTF-8", "de-DE", "fry", "frisian"] {
        assert_eq!(language_of_tag(tag), Language::English, "{tag}");
    }
}

#[test]
fn windows_language_ids_keep_only_their_primary_language() {
    assert_eq!(language_of_id(0x040c), Language::French);
    assert_eq!(language_of_id(0x0c0c), Language::French);
    assert_eq!(language_of_id(0x0409), Language::English);
    assert_eq!(language_of_id(0x0407), Language::English);
}

#[test]
fn an_explicit_choice_ignores_the_system() {
    assert_eq!(LanguageChoice::French.resolve(), Language::French);
    assert_eq!(LanguageChoice::English.resolve(), Language::English);
    assert_eq!(LanguageChoice::Auto.resolve(), system_language());
}

#[test]
fn each_language_choice_is_named_in_its_own_language() {
    let labels = LanguageChoice::ALL.map(LanguageChoice::label);
    assert_eq!(labels, ["Auto", "Français", "English"]);
}
