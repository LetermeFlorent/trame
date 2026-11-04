use trame::text::{BOLD, REGULAR};

use crate::{interface, system};

#[test]
fn empty_text_has_no_width() {
    let layout = system().layout("", interface(REGULAR));
    assert!(layout.glyphs.is_empty());
    assert_eq!(layout.width, 0.0);
}

#[test]
fn layout_width_grows_with_the_text() {
    let mut text = system();
    let short = text.layout("trame", interface(REGULAR));
    let long = text.layout("trame terminal", interface(REGULAR));
    assert!(long.width > short.width);
    assert!(short.cap_height > 0.0 && short.ascent > short.cap_height);
}

#[test]
fn layouts_are_cached_per_style() {
    let mut text = system();
    let regular = text.layout("Instances", interface(REGULAR));
    let again = text.layout("Instances", interface(REGULAR));
    let bold = text.layout("Instances", interface(BOLD));
    assert!(std::sync::Arc::ptr_eq(&regular, &again));
    assert!(!std::sync::Arc::ptr_eq(&regular, &bold));
    text.end_frame();
    let next_frame = text.layout("Instances", interface(REGULAR));
    assert!(std::sync::Arc::ptr_eq(&regular, &next_frame));
}
