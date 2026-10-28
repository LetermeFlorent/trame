use trame::ui::{FrameLayout, UiMetrics};

use crate::{input, text_system, workspace};

#[test]
fn an_unsaved_session_says_so_after_the_app_name() {
    let mut text = text_system();
    let workspace = workspace();
    let mut input = input(&workspace, UiMetrics::new(1.0));
    input.unsaved = true;
    let bar = FrameLayout::compute(&mut text, &input).title_bar;
    let (brand, _) = bar.brand.expect("a wide bar shows the app name");
    let shown = format!("trame {} (unsaved session)", env!("CARGO_PKG_VERSION"));
    let expected = text.layout(&shown, UiMetrics::new(1.0).label());
    assert_eq!(brand.width, expected.width);
}
