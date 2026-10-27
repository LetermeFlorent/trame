use trame::preferences::{Preferences, Updates};

#[test]
fn updates_install_by_themselves_by_default() {
    assert_eq!(Preferences::default().updates, Updates { auto: true });
}

#[test]
fn a_file_saved_with_the_old_launch_switches_gets_the_new_default() {
    let saved = r#"{"font_size":15.0,"update_at_launch":false,"update_ahead":true}"#;
    let preferences: Preferences = serde_json::from_str(saved).expect("parses");
    assert_eq!(preferences.updates, Updates::default());
}

#[test]
fn the_update_switch_is_saved_under_its_own_key() {
    let preferences = Preferences {
        updates: Updates { auto: false },
        ..Preferences::default()
    };
    let saved = serde_json::to_value(&preferences).expect("serializes");
    assert_eq!(saved["update_automatically"], false);
    assert!(saved.get("update_at_launch").is_none());
    let parsed: Preferences = serde_json::from_value(saved).expect("parses");
    assert_eq!(parsed.updates, preferences.updates);
}
