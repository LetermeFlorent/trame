use trame::preferences::{CONFIRM_FROM_TABS, Preferences};

#[test]
fn closing_several_tabs_asks_first_until_the_switch_is_off() {
    let base = Preferences::default();
    assert!(base.confirm_close);
    assert!(!base.confirms_closing(CONFIRM_FROM_TABS - 1));
    assert!(base.confirms_closing(CONFIRM_FROM_TABS));
    let off = base.toggle_confirm_close();
    assert!(!off.confirms_closing(CONFIRM_FROM_TABS + 3));
    assert_eq!(off.toggle_confirm_close(), base);
}

#[test]
fn the_switch_survives_json_and_older_files_ask_before_closing() {
    let saved = r#"{"font_size":14.0,"restore_text":true}"#;
    let older: Preferences = serde_json::from_str(saved).expect("parses");
    assert!(older.confirm_close);
    let off = Preferences::default().toggle_confirm_close();
    let json = serde_json::to_string(&off).expect("serializes");
    let parsed: Preferences = serde_json::from_str(&json).expect("parses");
    assert_eq!(parsed, off);
}
