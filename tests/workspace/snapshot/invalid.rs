use std::path::PathBuf;

use trame::preferences::Preferences;
use trame::session::SessionId;
use trame::workspace::{Snapshot, SnapshotError, TabSnapshot, Workspace};

use super::{pane, single, snapshot_of};

const MINIMAL: &str = r#"{"version":1,"sidebar_open":true,"font_size":14.0,"active_instance":0,"instances":[{"name":"a","active_pane":0,"root":{"Pane":{"tabs":[{}],"active":0}}}]}"#;

fn is_json_error(result: Result<Snapshot, SnapshotError>) -> bool {
    matches!(result, Err(SnapshotError::Json(_)))
}

#[test]
fn corrupt_json_is_an_error_not_a_panic() {
    for text in ["", "{", "not json", "{\"version\":1,", "\u{0}\u{0}\u{0}"] {
        assert!(is_json_error(Snapshot::from_json(text)), "{text:?}");
    }
}

#[test]
fn truncated_file_is_rejected() {
    let snapshot = snapshot_of(&single_workspace(), &[(1, "/home/ada")]);
    let text = snapshot.to_json().expect("serializes");
    let cut = &text[..text.len() / 2];
    assert!(is_json_error(Snapshot::from_json(cut)));
}

#[test]
fn right_version_with_missing_fields_is_rejected() {
    assert!(is_json_error(Snapshot::from_json(r#"{"version":1}"#)));
}

#[test]
fn wrong_field_types_are_rejected() {
    let text = MINIMAL.replace("14.0", "\"big\"");
    assert!(is_json_error(Snapshot::from_json(&text)));
}

#[test]
fn another_version_is_refused_with_its_number() {
    let text = MINIMAL.replace("\"version\":1", "\"version\":2");
    assert!(matches!(
        Snapshot::from_json(&text),
        Err(SnapshotError::Version(Some(2)))
    ));
}

#[test]
fn future_version_with_an_unknown_shape_is_still_a_version_error() {
    let result = Snapshot::from_json(r#"{"version":7,"layout":"other"}"#);
    assert!(matches!(result, Err(SnapshotError::Version(Some(7)))));
}

#[test]
fn missing_or_odd_version_is_refused() {
    for text in [
        r#"{"sidebar_open":true}"#,
        r#"{"version":"1"}"#,
        r#"{"version":-1}"#,
        "[]",
        "null",
    ] {
        assert!(
            matches!(Snapshot::from_json(text), Err(SnapshotError::Version(None))),
            "{text}"
        );
    }
}

#[test]
fn tab_without_a_directory_parses() {
    let snapshot = Snapshot::from_json(MINIMAL).expect("parses");
    let (workspace, seen) = super::restore(&snapshot);
    assert_eq!(workspace.tabs().len(), 1);
    assert_eq!(seen[0].cwd, None);
    assert_eq!(seen[0].title, None);
}

#[test]
fn directory_and_title_are_read_when_present() {
    let text = MINIMAL.replace("{}", r#"{"cwd":"/srv/app","title":"deploy"}"#);
    let snapshot = Snapshot::from_json(&text).expect("parses");
    let (_, seen) = super::restore(&snapshot);
    assert_eq!(seen[0].cwd, Some(PathBuf::from("/srv/app")));
    assert_eq!(seen[0].title.as_deref(), Some("deploy"));
}

#[test]
fn unknown_extra_fields_are_ignored() {
    let text = MINIMAL.replace("\"sidebar_open\"", "\"theme\":\"dark\",\"sidebar_open\"");
    assert!(Snapshot::from_json(&text).is_ok());
}

#[test]
fn empty_workspace_has_nothing_to_restore() {
    let snapshot = Workspace::new().snapshot(Preferences::default(), &|_| TabSnapshot::default());
    assert!(snapshot.instances.is_empty());
    let result = Workspace::restore(&snapshot, &mut |_| SessionId(1));
    assert!(matches!(result, Err(SnapshotError::Empty)));
}

#[test]
fn empty_workspace_round_trips_through_json() {
    let snapshot = Workspace::new().snapshot(Preferences::default(), &|_| TabSnapshot::default());
    let text = snapshot.to_json().expect("serializes");
    assert_eq!(Snapshot::from_json(&text).expect("parses"), snapshot);
}

#[test]
fn snapshot_with_only_empty_panes_has_nothing_to_restore() {
    let result = Workspace::restore(&single(pane(0)), &mut |_| SessionId(1));
    assert!(matches!(result, Err(SnapshotError::Empty)));
}

#[test]
fn errors_describe_themselves() {
    let json = Snapshot::from_json("{").unwrap_err();
    let version = Snapshot::from_json(r#"{"version":9}"#).unwrap_err();
    let empty = SnapshotError::Empty;
    for error in [json, version, empty] {
        assert!(!error.to_string().is_empty());
    }
}

fn single_workspace() -> Workspace {
    let mut workspace = Workspace::new();
    workspace.add_instance(None, SessionId(1));
    workspace
}
