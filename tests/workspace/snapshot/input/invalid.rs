use trame::workspace::{Snapshot, SnapshotError};

use super::MINIMAL;
use crate::snapshot::snapshot_of;
use crate::workspace_with;

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
    let snapshot = snapshot_of(&workspace_with(&[1]), &[(1, "/home/ada")]);
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
fn errors_describe_themselves() {
    let json = Snapshot::from_json("{").unwrap_err();
    let version = Snapshot::from_json(r#"{"version":9}"#).unwrap_err();
    let empty = SnapshotError::Empty;
    for error in [json, version, empty] {
        assert!(!error.to_string().is_empty());
    }
}
