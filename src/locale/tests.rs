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

fn differ(pick: fn(&Strings) -> &'static str) {
    let english = pick(Language::English.strings());
    let french = pick(Language::French.strings());
    assert_ne!(english, french);
}

#[test]
fn each_language_has_its_own_update_and_menu_texts() {
    let picks: [fn(&Strings) -> &'static str; 8] = [
        |s| s.checking_updates,
        |s| s.downloading_version,
        |s| s.installing_version,
        |s| s.update_to,
        |s| s.rename,
        |s| s.change_color,
        |s| s.change_icon,
        |s| s.change_folder,
    ];
    picks.into_iter().for_each(differ);
}

#[test]
fn each_language_has_its_own_settings_texts() {
    let picks: [fn(&Strings) -> &'static str; 19] = [
        |s| s.settings,
        |s| s.at_launch,
        |s| s.remember_instances,
        |s| s.remember_tabs,
        |s| s.restore_text,
        |s| s.terminals,
        |s| s.startup_command,
        |s| s.instance_tabs,
        |s| s.add,
        |s| s.display,
        |s| s.text_size,
        |s| s.interface_size,
        |s| s.language,
        |s| s.folders,
        |s| s.default_folder,
        |s| s.choose,
        |s| s.pick_folder,
        |s| s.connections,
        |s| s.no_connections,
    ];
    picks.into_iter().for_each(differ);
}

#[test]
fn each_language_has_its_own_ssh_texts() {
    let picks: [fn(&Strings) -> &'static str; 12] = [
        |s| s.ssh_connection,
        |s| s.general,
        |s| s.connections_tab,
        |s| s.change_connection,
        |s| s.save,
        |s| s.cancel,
        |s| s.servers,
        |s| s.new_connection,
        |s| s.ssh_address,
        |s| s.ssh_user,
        |s| s.ssh_password,
        |s| s.connect,
    ];
    picks.into_iter().for_each(differ);
}

#[test]
fn each_language_has_its_own_closing_texts() {
    let picks: [fn(&Strings) -> &'static str; 5] = [
        |s| s.confirm_close,
        |s| s.close_window,
        |s| s.close_instance,
        |s| s.tabs_will_close,
        |s| s.close,
    ];
    picks.into_iter().for_each(differ);
}

#[test]
fn each_language_has_its_own_folder_window_texts() {
    let picks: [fn(&Strings) -> &'static str; 3] = [|s| s.change_folder, |s| s.change, |s| s.apply];
    picks.into_iter().for_each(differ);
}

#[test]
fn each_language_has_its_own_custom_color_texts() {
    let picks: [fn(&Strings) -> &'static str; 3] =
        [|s| s.my_colors, |s| s.new_color, |s| s.edit_color];
    picks.into_iter().for_each(differ);
}

#[test]
fn each_language_has_its_own_custom_icon_texts() {
    let picks: [fn(&Strings) -> &'static str; 2] = [|s| s.letters_or_image, |s| s.pick_image];
    picks.into_iter().for_each(differ);
}

#[test]
fn each_language_choice_is_named_in_its_own_language() {
    let labels = LanguageChoice::ALL.map(LanguageChoice::label);
    assert_eq!(labels, ["Auto", "Français", "English"]);
}
