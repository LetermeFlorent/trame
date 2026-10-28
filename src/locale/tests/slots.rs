use crate::locale::{
    ACTION_SLOT, ALIAS_SLOT, COUNT_SLOT, Language, NUMBER_SLOT, Strings, VERSION_SLOT,
};

fn slotted(strings: &Strings) -> [(&'static str, &'static str); 11] {
    [
        (strings.instance_name, NUMBER_SLOT),
        (strings.downloading_version, VERSION_SLOT),
        (strings.installing_version, VERSION_SLOT),
        (strings.update_to, VERSION_SLOT),
        (strings.restart_into, VERSION_SLOT),
        (strings.update_question, VERSION_SLOT),
        (strings.restart_question, VERSION_SLOT),
        (strings.remove_question, ALIAS_SLOT),
        (strings.tabs_will_close, COUNT_SLOT),
        (strings.lines_in_all, COUNT_SLOT),
        (strings.keys.taken_by, ACTION_SLOT),
    ]
}

#[test]
fn every_phrase_that_takes_a_value_says_where_it_goes() {
    for language in [Language::English, Language::French] {
        for (phrase, slot) in slotted(language.strings()) {
            let found = phrase.matches(slot).count();
            assert_eq!(found, 1, "{language:?} {slot} in {phrase:?}");
        }
    }
}
