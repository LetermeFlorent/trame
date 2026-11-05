use std::path::PathBuf;

use trame::ui::SidebarMode;
use trame::workspace::Snapshot;

use super::MINIMAL;

#[test]
fn tab_without_a_directory_parses() {
    let snapshot = Snapshot::from_json(MINIMAL).expect("parses");
    let (workspace, seen) = crate::snapshot::restore(&snapshot);
    assert_eq!(workspace.tabs().len(), 1);
    assert_eq!(seen[0].cwd, None);
    assert_eq!(seen[0].title, None);
}

#[test]
fn directory_and_title_are_read_when_present() {
    let text = MINIMAL.replace("{}", r#"{"cwd":"/srv/app","title":"deploy"}"#);
    let snapshot = Snapshot::from_json(&text).expect("parses");
    let (_, seen) = crate::snapshot::restore(&snapshot);
    assert_eq!(seen[0].cwd, Some(PathBuf::from("/srv/app")));
    assert_eq!(seen[0].title.as_deref(), Some("deploy"));
}

#[test]
fn unknown_extra_fields_are_ignored() {
    let text = MINIMAL.replace("\"sidebar_open\"", "\"theme\":\"dark\",\"sidebar_open\"");
    assert!(Snapshot::from_json(&text).is_ok());
}

#[test]
fn an_older_open_flag_picks_the_open_panel_or_the_rail() {
    for (open, mode) in [(true, SidebarMode::Open), (false, SidebarMode::Rail)] {
        let flag = format!("\"sidebar_open\":{open}");
        let text = MINIMAL.replace("\"sidebar_open\":true", &flag);
        let snapshot = Snapshot::from_json(&text).expect("an older file parses");
        let (workspace, _) = crate::snapshot::restore(&snapshot);
        assert_eq!(workspace.sidebar, mode);
    }
}

#[test]
fn a_hidden_panel_survives_a_restart() {
    let mut snapshot = Snapshot::from_json(MINIMAL).expect("parses");
    snapshot.sidebar = SidebarMode::Hidden;
    let (workspace, _) = crate::snapshot::restore(&crate::snapshot::through_json(&snapshot));
    assert_eq!(workspace.sidebar, SidebarMode::Hidden);
}

#[test]
fn a_new_file_still_tells_older_versions_whether_the_panel_is_open() {
    let modes = [
        (SidebarMode::Open, true),
        (SidebarMode::Rail, false),
        (SidebarMode::Hidden, false),
    ];
    let mut snapshot = Snapshot::from_json(MINIMAL).expect("parses");
    for (mode, open) in modes {
        snapshot.sidebar = mode;
        let text = snapshot.to_json().expect("serializes");
        let value: serde_json::Value = serde_json::from_str(&text).expect("valid json");
        assert_eq!(value["sidebar_open"], serde_json::Value::Bool(open));
        assert_eq!(Snapshot::from_json(&text).expect("parses").sidebar, mode);
    }
}

#[test]
fn the_new_field_wins_over_the_old_flag() {
    let text = MINIMAL.replace(
        "\"sidebar_open\"",
        "\"sidebar\":\"Hidden\",\"sidebar_open\"",
    );
    let snapshot = Snapshot::from_json(&text).expect("parses");
    assert_eq!(snapshot.sidebar, SidebarMode::Hidden);
}
