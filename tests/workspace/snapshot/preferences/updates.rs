use trame::preferences::Preferences;
use trame::workspace::Snapshot;

const OLD_KEYS: [&str; 3] = ["update_automatically", "update_at_launch", "update_ahead"];

#[test]
fn a_file_saved_with_the_old_update_switches_still_loads() {
    let saved = r#"{"font_size":15.0,"update_automatically":false,"update_at_launch":false,"update_ahead":true}"#;
    let preferences: Preferences = serde_json::from_str(saved).expect("parses");
    assert_eq!(preferences, Preferences::default().with_font_size(15.0));
}

#[test]
fn a_session_saved_with_the_update_switch_still_restores() {
    let saved = r#"{"version":1,"sidebar_open":true,"font_size":16.0,"update_automatically":true,"active_instance":0,"instances":[{"name":"a","active_pane":0,"root":{"Pane":{"tabs":[{}],"active":0}}}]}"#;
    let snapshot = Snapshot::from_json(saved).expect("parses");
    assert_eq!(
        snapshot.preferences,
        Preferences::default().with_font_size(16.0)
    );
}

#[test]
fn no_update_switch_is_saved_any_more() {
    let saved = serde_json::to_value(Preferences::default()).expect("serializes");
    for key in OLD_KEYS {
        assert!(saved.get(key).is_none(), "{key} is still saved");
    }
}
