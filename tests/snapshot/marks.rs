use trame::session::SessionId;
use trame::theme::Swatch;
use trame::workspace::{Snapshot, Workspace};

use crate::{FIRST_RESTORED_ID, restore, snapshot_of};

const OLD_FILE: &str = r#"{"version":1,"sidebar_open":true,"font_size":14.0,"active_instance":0,"instances":[{"name":"old","active_pane":0,"root":{"Pane":{"tabs":[{"cwd":"/home/ada"},{}],"active":0}}}]}"#;

fn marked() -> Workspace {
    let mut workspace = Workspace::new();
    workspace.add_instance(Some("site".to_owned()), SessionId(1));
    workspace.add_tab(SessionId(2));
    workspace.add_instance(None, SessionId(3));
    workspace.rename_tab(SessionId(2), "server");
    workspace.set_tab_color(SessionId(2), Some(Swatch::Purple));
    workspace.set_tab_color(SessionId(3), Some(Swatch::Green));
    let first = workspace.instances[0].id;
    workspace.set_instance_color(first, Some(Swatch::Orange));
    workspace
}

fn restored_id(position: u64) -> SessionId {
    SessionId(FIRST_RESTORED_ID + position)
}

#[test]
fn names_and_colours_survive_a_json_round_trip() {
    let snapshot = snapshot_of(&marked(), &[]);
    let text = snapshot.to_json().expect("serializes");
    assert_eq!(Snapshot::from_json(&text).expect("parses"), snapshot);
}

#[test]
fn restoring_brings_back_every_name_and_colour() {
    let text = snapshot_of(&marked(), &[]).to_json().expect("serializes");
    let snapshot = Snapshot::from_json(&text).expect("parses");
    let (workspace, _) = restore(&snapshot);
    assert_eq!(workspace.instances[0].color, Some(Swatch::Orange));
    assert_eq!(workspace.instances[1].color, None);
    assert_eq!(workspace.tab_name(restored_id(1)), None);
    assert_eq!(workspace.tab_name(restored_id(2)), Some("server"));
    assert_eq!(workspace.tab_color(restored_id(2)), Some(Swatch::Purple));
    assert_eq!(workspace.tab_color(restored_id(3)), Some(Swatch::Green));
}

#[test]
fn a_session_without_marks_writes_no_extra_keys() {
    let mut workspace = Workspace::new();
    workspace.add_instance(None, SessionId(1));
    let text = snapshot_of(&workspace, &[]).to_json().expect("serializes");
    assert!(!text.contains("color"));
    assert!(!text.contains("title"));
}

#[test]
fn an_old_session_file_still_loads_without_marks() {
    let snapshot = Snapshot::from_json(OLD_FILE).expect("old files load");
    let (workspace, seen) = restore(&snapshot);
    assert_eq!(workspace.instances[0].name, "old");
    assert_eq!(workspace.instances[0].color, None);
    assert_eq!(seen.len(), 2);
    assert_eq!(workspace.tab_name(restored_id(1)), None);
    assert_eq!(workspace.tab_color(restored_id(1)), None);
}

#[test]
fn a_blank_saved_title_is_not_a_name() {
    let text = OLD_FILE.replace("{}]", r#"{"title":"   "}]"#);
    let snapshot = Snapshot::from_json(&text).expect("parses");
    let (workspace, _) = restore(&snapshot);
    assert_eq!(workspace.tab_name(restored_id(2)), None);
}

#[test]
fn an_unknown_colour_name_is_a_parse_error() {
    let text = OLD_FILE.replace(r#""name":"old""#, r#""name":"old","color":"mauve""#);
    assert!(Snapshot::from_json(&text).is_err());
}
