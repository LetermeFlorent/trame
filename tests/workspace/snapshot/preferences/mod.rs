mod closing;
mod history;
mod launch;
mod links;
mod persistence;
mod sizes;
mod ssh;
mod updates;

use trame::locale::LanguageChoice;
use trame::preferences::Preferences;
use trame::ui::{Density, PanelSide};
use trame::workspace::Snapshot;

const OLDER_FILE: &str = r#"{"version":1,"sidebar_open":true,"font_size":16.0,"active_instance":0,"instances":[{"name":"a","active_pane":0,"root":{"Pane":{"tabs":[{}],"active":0}}}]}"#;

#[test]
fn older_files_get_the_default_preferences() {
    let snapshot = Snapshot::from_json(OLDER_FILE).expect("parses");
    assert_eq!(
        snapshot.preferences,
        Preferences::default().with_font_size(16.0)
    );
    assert!(snapshot.preferences.restore_text);
    assert_eq!(snapshot.preferences.default_folder, None);
}

#[test]
fn preferences_saved_before_the_language_choice_follow_the_system() {
    let saved = r#"{"font_size":15.0,"restore_text":true,"interface_size":110}"#;
    let preferences: Preferences = serde_json::from_str(saved).expect("parses");
    assert_eq!(preferences.language, LanguageChoice::Auto);
    assert_eq!(preferences.interface_size, 110);
    assert_eq!(preferences.panel_side, PanelSide::Left);
    assert_eq!(preferences.density, Density::Compact);
}
