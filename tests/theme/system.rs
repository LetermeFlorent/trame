use std::cell::RefCell;

use trame::theme::Appearance;
use trame::theme::system::{detect_with, parse_color_scheme, parse_gtk_theme};

fn reader<'a>(
    color_scheme: Option<&'a str>,
    gtk_theme: Option<&'a str>,
    asked: &'a RefCell<Vec<String>>,
) -> impl Fn(&str) -> Option<String> + 'a {
    move |key| {
        asked.borrow_mut().push(key.to_string());
        let value = match key {
            "color-scheme" => color_scheme,
            "gtk-theme" => gtk_theme,
            _ => None,
        };
        value.map(str::to_string)
    }
}

#[test]
fn prefer_dark_color_scheme_means_dark() {
    assert_eq!(
        parse_color_scheme("'prefer-dark'\n"),
        Some(Appearance::Dark)
    );
}

#[test]
fn prefer_light_color_scheme_means_light() {
    assert_eq!(
        parse_color_scheme("'prefer-light'\n"),
        Some(Appearance::Light)
    );
}

#[test]
fn default_color_scheme_gives_no_answer() {
    assert_eq!(parse_color_scheme("'default'\n"), None);
    assert_eq!(parse_color_scheme(""), None);
    assert_eq!(parse_color_scheme("''\n"), None);
}

#[test]
fn typed_empty_strings_are_understood() {
    assert_eq!(parse_color_scheme("@s ''\n"), None);
    assert_eq!(parse_gtk_theme("@s ''\n"), None);
}

#[test]
fn gtk_theme_with_dark_in_its_name_means_dark() {
    assert_eq!(parse_gtk_theme("'Adwaita-dark'\n"), Some(Appearance::Dark));
    assert_eq!(parse_gtk_theme("'Yaru-DARK'\n"), Some(Appearance::Dark));
}

#[test]
fn other_gtk_themes_mean_light() {
    assert_eq!(parse_gtk_theme("'Adwaita'\n"), Some(Appearance::Light));
    assert_eq!(parse_gtk_theme("'Yaru'\n"), Some(Appearance::Light));
}

#[test]
fn empty_gtk_theme_gives_no_answer() {
    assert_eq!(parse_gtk_theme("''\n"), None);
    assert_eq!(parse_gtk_theme("\n"), None);
}

#[test]
fn color_scheme_wins_and_the_gtk_theme_is_not_asked() {
    let asked = RefCell::new(Vec::new());
    let read = reader(Some("'prefer-dark'\n"), Some("'Adwaita'\n"), &asked);
    assert_eq!(detect_with(read), Some(Appearance::Dark));
    assert_eq!(*asked.borrow(), ["color-scheme"]);
}

#[test]
fn the_gtk_theme_answers_when_the_color_scheme_does_not() {
    let asked = RefCell::new(Vec::new());
    let read = reader(Some("'default'\n"), Some("'Adwaita-dark'\n"), &asked);
    assert_eq!(detect_with(read), Some(Appearance::Dark));
    assert_eq!(*asked.borrow(), ["color-scheme", "gtk-theme"]);
}

#[test]
fn missing_gsettings_gives_no_answer() {
    let asked = RefCell::new(Vec::new());
    assert_eq!(detect_with(reader(None, None, &asked)), None);
}
