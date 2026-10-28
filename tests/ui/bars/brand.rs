use trame::ui::{FrameLayout, UiMetrics};

use crate::{input, text_system, workspace};

fn shows(unsaved: bool, update_failed: bool, note: &str) {
    let mut text = text_system();
    let workspace = workspace();
    let mut input = input(&workspace, UiMetrics::new(1.0));
    input.unsaved = unsaved;
    input.update_failed = update_failed;
    let bar = FrameLayout::compute(&mut text, &input).title_bar;
    let (brand, _) = bar.brand.expect("a wide bar shows the app name");
    let shown = format!("trame {} ({note})", env!("CARGO_PKG_VERSION"));
    let expected = text.layout(&shown, UiMetrics::new(1.0).label());
    assert_eq!(brand.width, expected.width);
}

#[test]
fn an_unsaved_session_says_so_after_the_app_name() {
    shows(true, false, "unsaved session");
}

#[test]
fn a_failed_update_says_so_after_the_app_name() {
    shows(false, true, "update failed");
}

#[test]
fn both_notes_share_one_pair_of_brackets() {
    shows(true, true, "unsaved session, update failed");
}
