use trame::preferences::Preferences;

#[test]
fn smooth_scrolling_is_on_by_default_and_for_older_files() {
    assert!(Preferences::default().smooth_scroll);
    let older: Preferences = serde_json::from_str(r#"{"font_size":14.0}"#).expect("parses");
    assert!(older.smooth_scroll);
    let off = Preferences {
        smooth_scroll: false,
        ..Preferences::default()
    };
    let saved = serde_json::to_value(&off).expect("serializes");
    let read: Preferences = serde_json::from_value(saved).expect("parses");
    assert!(!read.smooth_scroll);
}
