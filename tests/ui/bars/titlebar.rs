use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use trame::files::{home_dir, short_path};
use trame::text::{TextStyle, TextSystem};
use trame::ui::frame::{TitleBarFrame, fit_text_start};
use trame::ui::{FrameLayout, UiMetrics};

use crate::{WIDTH, input, text_system, workspace};

const NARROW: f32 = 420.0;
const ELLIPSIS: &str = "\u{2026}";

fn bar(text: &mut TextSystem, folder: Option<&Path>, width: f32) -> TitleBarFrame {
    let workspace = workspace();
    let mut input = input(&workspace, UiMetrics::new(1.0));
    input.folder = folder;
    input.width = width;
    FrameLayout::compute(text, &input).title_bar
}

fn label() -> TextStyle {
    UiMetrics::new(1.0).label()
}

fn outside_home() -> PathBuf {
    match cfg!(windows) {
        true => PathBuf::from(r"Z:\work\trame"),
        false => PathBuf::from("/srv/work/trame"),
    }
}

#[test]
fn the_app_name_and_version_sit_in_the_middle_of_the_bar() {
    let mut text = text_system();
    let bar = bar(&mut text, None, WIDTH);
    let (brand, x) = bar.brand.expect("a wide bar shows the app name");
    let expected = text.layout(concat!("trame ", env!("CARGO_PKG_VERSION")), label());
    assert_eq!(brand.width, expected.width);
    assert!((x + brand.width / 2.0 - bar.rect.center().x).abs() <= 1.0);
}

#[test]
fn a_narrow_bar_drops_the_app_name_and_keeps_the_instance_name() {
    let mut text = text_system();
    let bar = bar(&mut text, None, NARROW);
    assert!(bar.brand.is_none());
    assert!(bar.title.is_some());
}

#[test]
fn the_terminal_folder_follows_the_instance_name() {
    let mut text = text_system();
    let folder = outside_home();
    let bar = bar(&mut text, Some(&folder), WIDTH);
    let (name, name_x) = bar.title.expect("the instance name is shown");
    let (shown, x) = bar.subtitle.expect("the folder is shown");
    let (_, brand_x) = bar.brand.expect("the app name is shown");
    assert!(x > name_x + name.width);
    assert!(x + shown.width <= brand_x);
    let whole = text.layout(&folder.display().to_string(), label());
    assert_eq!(shown.width, whole.width);
}

#[test]
fn a_folder_inside_home_is_shown_from_the_tilde() {
    let mut text = text_system();
    let folder = home_dir()
        .expect("the home folder is known")
        .join("Projets");
    let bar = bar(&mut text, Some(&folder), WIDTH);
    let (shown, _) = bar.subtitle.expect("the folder is shown");
    let short = text.layout(&format!("~{MAIN_SEPARATOR}Projets"), label());
    assert_eq!(shown.width, short.width);
}

#[test]
fn without_a_folder_the_bar_shows_only_the_instance_name() {
    let mut text = text_system();
    assert!(bar(&mut text, None, WIDTH).subtitle.is_none());
}

#[test]
fn a_long_folder_is_cut_before_it_reaches_the_app_name() {
    let mut text = text_system();
    let deep = (0..30).fold(outside_home(), |path, index| {
        path.join(format!("level{index}"))
    });
    let bar = bar(&mut text, Some(&deep), WIDTH);
    let (shown, x) = bar.subtitle.expect("the folder is shown");
    let (_, brand_x) = bar.brand.expect("the app name is shown");
    assert!(x + shown.width <= brand_x);
    let whole = text.layout(&deep.display().to_string(), label());
    assert!(shown.width < whole.width);
}

#[test]
fn cutting_from_the_start_keeps_the_end_of_the_path() {
    let mut text = text_system();
    let kept = format!("{ELLIPSIS}fourth/fifth");
    let width = text.layout(&kept, label()).width;
    let fitted = fit_text_start(&mut text, "first/second/third/fourth/fifth", label(), width);
    assert_eq!(fitted, kept);
}

#[test]
fn a_label_that_fits_is_left_whole() {
    let mut text = text_system();
    assert_eq!(fit_text_start(&mut text, "short", label(), 1000.0), "short");
    assert_eq!(fit_text_start(&mut text, "", label(), 0.0), "");
    assert_eq!(fit_text_start(&mut text, "short", label(), 0.0), ELLIPSIS);
}

#[test]
fn short_paths_replace_the_home_folder_with_a_tilde() {
    let home = outside_home();
    let logs = home.join("logs");
    let parent = home.parent().expect("the fake home has a parent");
    assert_eq!(short_path(&home, Some(&home)), "~");
    assert_eq!(
        short_path(&logs, Some(&home)),
        format!("~{MAIN_SEPARATOR}logs")
    );
    assert_eq!(
        short_path(parent, Some(&home)),
        parent.display().to_string()
    );
    assert_eq!(short_path(&logs, None), logs.display().to_string());
}
