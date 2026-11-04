use trame::preferences::Preferences;

#[test]
fn link_detection_is_on_by_default_and_for_older_files() {
    assert!(Preferences::default().detect_links);
    let older: Preferences = serde_json::from_str(r#"{"font_size":14.0}"#).expect("parses");
    assert!(older.detect_links);
    let off = Preferences {
        detect_links: false,
        ..Preferences::default()
    };
    let saved = serde_json::to_value(&off).expect("serializes");
    assert_eq!(saved["detect_links"], false);
    let read: Preferences = serde_json::from_value(saved).expect("parses");
    assert!(!read.detect_links);
}
