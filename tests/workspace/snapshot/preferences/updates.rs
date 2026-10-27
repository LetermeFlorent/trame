use trame::preferences::{Preferences, Updates};

#[test]
fn updates_wait_for_a_click_and_download_ahead_by_default() {
    let updates = Preferences::default().updates;
    assert_eq!(
        updates,
        Updates {
            auto: false,
            ahead: true,
        }
    );
    assert!(!updates.downloads_ahead());
}

#[test]
fn a_file_saved_before_the_update_switches_gets_the_defaults() {
    let saved = r#"{"font_size":15.0,"efficiency":true}"#;
    let preferences: Preferences = serde_json::from_str(saved).expect("parses");
    assert_eq!(preferences.updates, Updates::default());
}

#[test]
fn the_update_switches_are_saved_under_their_own_keys() {
    let preferences = Preferences {
        updates: Updates {
            auto: true,
            ahead: false,
        },
        ..Preferences::default()
    };
    let saved = serde_json::to_value(&preferences).expect("serializes");
    assert_eq!(saved["update_at_launch"], true);
    assert_eq!(saved["update_ahead"], false);
    let parsed: Preferences = serde_json::from_value(saved).expect("parses");
    assert_eq!(parsed.updates, preferences.updates);
    assert!(!parsed.updates.downloads_ahead());
    let both = Updates {
        auto: true,
        ahead: true,
    };
    assert!(both.downloads_ahead());
}
