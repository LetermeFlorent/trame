use std::path::PathBuf;

use trame::locale::Language;
use trame::preferences::Preferences;
use trame::ui::Hit;
use trame::ui::frame::SettingControl;

use super::{english, panel, with_settings};
use crate::text_system;

fn with_folder(folder: &str) -> Preferences {
    Preferences {
        default_folder: Some(PathBuf::from(folder)),
        ..Preferences::default()
    }
}

#[test]
fn without_a_default_folder_the_row_only_offers_to_choose_one() {
    let mut text = text_system();
    let english = Language::English.strings();
    let layout = with_settings(&mut text, &Preferences::default(), english);
    let row = &panel(&layout).folder;
    assert_eq!(row.chips.len(), 1);
    let chip = &row.chips[0];
    assert_eq!(chip.control, SettingControl::DefaultFolder);
    assert!(!chip.selected);
    let expected = text.layout(english.choose, layout.metrics.label());
    assert_eq!(chip.label.width, expected.width);
    assert!(row.clear.is_none());
    assert_eq!(
        layout.hit(chip.rect.center(), false),
        Some(Hit::Setting(SettingControl::DefaultFolder))
    );
}

#[test]
fn a_chosen_folder_shows_its_name_and_a_button_to_forget_it() {
    let mut text = text_system();
    let preferences = with_folder("C:/work/projects");
    let layout = with_settings(&mut text, &preferences, Language::English.strings());
    let row = &panel(&layout).folder;
    let chip = &row.chips[0];
    let expected = text.layout("projects", layout.metrics.label());
    assert_eq!(chip.label.width, expected.width);
    let (clear, control) = row.clear.expect("a button to forget the folder");
    assert_eq!(control, SettingControl::ClearFolder);
    assert!(chip.rect.right() < clear.x);
    assert!(clear.right() <= row.line.rect.right());
    assert_eq!(
        layout.hit(clear.center(), false),
        Some(Hit::Setting(SettingControl::ClearFolder))
    );
}

#[test]
fn a_long_folder_name_is_shortened_beside_its_label() {
    let layout = english(&with_folder(&"long folder name ".repeat(12)));
    let row = &panel(&layout).folder;
    let chip = &row.chips[0];
    assert!(chip.rect.x > row.line.label_x + row.line.label.width);
    assert!(chip.label.width < chip.rect.width);
}
