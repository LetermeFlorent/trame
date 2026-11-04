use std::path::PathBuf;

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
