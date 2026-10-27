use std::path::{Path, PathBuf};

use trame::locale::Language;
use trame::preferences::Preferences;
use trame::text::TextSystem;
use trame::ui::FrameLayout;
use trame::ui::Hit;
use trame::ui::frame::{ChoiceRow, SettingControl, SettingsRequest, fit_text_start};

use super::{english, general_request, page, placed, with_settings};
use crate::text_system;

const CURRENT: &str = "D:/shell/here";

fn with_folder(folder: &str) -> Preferences {
    Preferences {
        default_folder: Some(PathBuf::from(folder)),
        ..Preferences::default()
    }
}

fn folder(layout: &FrameLayout) -> &ChoiceRow {
    let row = page(layout).row(SettingControl::DefaultFolder);
    row.expect("the folder row")
}

fn opened_in(text: &mut TextSystem, preferences: &Preferences, current: &str) -> FrameLayout {
    let request = SettingsRequest {
        folder: Some(Path::new(current)),
        ..general_request(preferences, Language::English.strings())
    };
    placed(text, &request)
}

#[test]
fn without_a_default_folder_the_row_only_offers_to_choose_one() {
    let mut text = text_system();
    let english = Language::English.strings();
    let layout = with_settings(&mut text, &Preferences::default(), english);
    let row = folder(&layout);
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
fn a_chosen_folder_shows_its_path_and_a_button_to_forget_it() {
    let mut text = text_system();
    let preferences = with_folder("C:/work/projects");
    let layout = opened_in(&mut text, &preferences, CURRENT);
    let row = folder(&layout);
    let chip = &row.chips[0];
    let expected = text.layout("C:/work/projects", layout.metrics.label());
    assert_eq!(chip.label.width, expected.width);
    assert!(!chip.faint);
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
    let row = folder(&layout);
    let chip = &row.chips[0];
    assert!(chip.rect.x > row.line.label_x + row.line.label.width);
    assert!(chip.label.width < chip.rect.width);
}

#[test]
fn without_a_choice_the_row_shows_the_terminal_folder_greyed() {
    let mut text = text_system();
    let layout = opened_in(&mut text, &Preferences::default(), CURRENT);
    let row = folder(&layout);
    let chip = &row.chips[0];
    let expected = text.layout(CURRENT, layout.metrics.label());
    assert_eq!(chip.label.width, expected.width);
    assert_eq!(chip.control, SettingControl::DefaultFolder);
    assert!(chip.faint);
    assert!(row.clear.is_none());
}

#[test]
fn the_choose_button_is_not_greyed() {
    let layout = english(&Preferences::default());
    assert!(!folder(&layout).chips[0].faint);
}

#[test]
fn a_long_path_keeps_its_end_in_the_chip() {
    let mut text = text_system();
    let deep = format!("C:/{}final", "nested/".repeat(30));
    let layout = opened_in(&mut text, &with_folder(&deep), CURRENT);
    let row = folder(&layout);
    let chip = &row.chips[0];
    let style = layout.metrics.label();
    let whole = text.layout(&deep, style);
    assert!(chip.label.width < whole.width);
    let fitted = fit_text_start(&mut text, &deep, style, chip.label.width);
    assert!(fitted.ends_with("nested/final"));
    assert_eq!(text.layout(&fitted, style).width, chip.label.width);
}
