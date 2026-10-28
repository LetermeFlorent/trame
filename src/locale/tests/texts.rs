use crate::locale::{Language, Strings};

fn differ(pick: fn(&Strings) -> &'static str) {
    let english = pick(Language::English.strings());
    let french = pick(Language::French.strings());
    assert_ne!(english, french);
}

#[test]
fn each_language_has_its_own_update_and_menu_texts() {
    let picks: [fn(&Strings) -> &'static str; 11] = [
        |s| s.checking_updates,
        |s| s.downloading_version,
        |s| s.installing_version,
        |s| s.update_to,
        |s| s.restart_into,
        |s| s.rename,
        |s| s.change_color,
        |s| s.change_icon,
        |s| s.change_folder,
        |s| s.unsaved_session,
        |s| s.shell_exited_at_once,
    ];
    picks.into_iter().for_each(differ);
}

#[test]
fn each_language_has_its_own_settings_texts() {
    let picks: [fn(&Strings) -> &'static str; 29] = [
        |s| s.settings,
        |s| s.at_launch,
        |s| s.remember_instances,
        |s| s.remember_tabs,
        |s| s.restore_text,
        |s| s.terminals,
        |s| s.startup_command,
        |s| s.instance_tabs,
        |s| s.history_lines,
        |s| s.add,
        |s| s.display,
        |s| s.text_size,
        |s| s.interface_size,
        |s| s.language,
        |s| s.panel_side,
        |s| s.left,
        |s| s.right,
        |s| s.spacing,
        |s| s.spacious,
        |s| s.folders,
        |s| s.default_folder,
        |s| s.choose,
        |s| s.pick_folder,
        |s| s.connections,
        |s| s.no_connections,
        |s| s.system_section,
        |s| s.rest_hidden_tabs,
        |s| s.updates,
        |s| s.update_automatically,
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
    let picks: [fn(&Strings) -> &'static str; 9] = [
        |s| s.confirm_close,
        |s| s.close_window,
        |s| s.close_instance,
        |s| s.tabs_will_close,
        |s| s.close,
        |s| s.update_question,
        |s| s.restart_question,
        |s| s.update_restarts,
        |s| s.restart,
    ];
    picks.into_iter().for_each(differ);
}

#[test]
fn each_language_has_its_own_paste_texts() {
    let picks: [fn(&Strings) -> &'static str; 5] = [
        |s| s.confirm_paste,
        |s| s.paste_question,
        |s| s.lines_in_all,
        |s| s.hidden_removed,
        |s| s.paste,
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
fn each_language_has_its_own_forget_texts() {
    let picks: [fn(&Strings) -> &'static str; 3] =
        [|s| s.remove_question, |s| s.hand_written, |s| s.remove];
    picks.into_iter().for_each(differ);
}
