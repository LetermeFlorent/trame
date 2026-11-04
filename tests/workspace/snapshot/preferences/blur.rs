use trame::preferences::Preferences;

#[test]
fn the_blur_switch_is_saved_under_its_own_key() {
    let on = Preferences {
        blur: true,
        ..Preferences::default()
    };
    let saved = serde_json::to_value(&on).expect("serializes");
    assert_eq!(saved["blur"], true);
    assert!(saved.get("mica").is_none());
    let read: Preferences = serde_json::from_value(saved).expect("parses");
    assert_eq!(read, on);
    let off = serde_json::to_value(Preferences::default()).expect("serializes");
    assert_eq!(off["blur"], false);
}
