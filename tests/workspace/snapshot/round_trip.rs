use std::path::PathBuf;

use trame::session::SessionId;
use trame::workspace::{Edge, Snapshot, WindowSnapshot, Workspace};

use super::{FIRST_RESTORED_ID, restore, shape, snapshot_of};

fn sample() -> Workspace {
    let mut workspace = Workspace::new();
    workspace.add_instance(Some("build".to_owned()), SessionId(1));
    workspace.add_tab(SessionId(2));
    workspace.split_active(SessionId(3), Edge::Right);
    workspace.add_instance(Some("notes".to_owned()), SessionId(4));
    workspace.active = 0;
    workspace
}

#[test]
fn json_round_trip_is_lossless() {
    let snapshot = snapshot_of(&sample(), &[(1, "/home/ada"), (3, "/tmp")]);
    let text = snapshot.to_json().expect("serializes");
    assert_eq!(Snapshot::from_json(&text).expect("parses"), snapshot);
}

#[test]
fn restore_rebuilds_names_tabs_and_selection() {
    let snapshot = snapshot_of(&sample(), &[(1, "/home/ada"), (3, "/tmp")]);
    let (workspace, _) = restore(&snapshot);
    assert_eq!(workspace.instances.len(), 2);
    assert_eq!(workspace.instances[0].name, "build");
    assert_eq!(workspace.instances[1].name, "notes");
    assert_eq!(workspace.active, 0);
    assert_eq!(shape(&workspace.instances[0]), "Row(1:[2],1:[1])");
    assert_eq!(
        workspace.active_tab(),
        Some(SessionId(FIRST_RESTORED_ID + 3))
    );
}

#[test]
fn each_pane_keeps_its_own_active_tab() {
    let (workspace, _) = restore(&snapshot_of(&sample(), &[]));
    let instance = &workspace.instances[0];
    let first = instance.root.panes()[0];
    assert_eq!(instance.panes[&first].active, 1);
    assert_eq!(
        instance.panes[&first].active_tab(),
        Some(SessionId(FIRST_RESTORED_ID + 2))
    );
}

#[test]
fn directories_reach_the_spawn_callback_in_tab_order() {
    let snapshot = snapshot_of(&sample(), &[(1, "/home/ada"), (3, "/tmp")]);
    let (workspace, seen) = restore(&snapshot);
    let cwds: Vec<Option<PathBuf>> = seen.into_iter().map(|tab| tab.cwd).collect();
    assert_eq!(
        cwds,
        vec![
            Some(PathBuf::from("/home/ada")),
            None,
            Some(PathBuf::from("/tmp")),
            None
        ]
    );
    assert_eq!(workspace.tabs().len(), 4);
}

#[test]
fn restored_tabs_get_fresh_session_ids() {
    let (workspace, _) = restore(&snapshot_of(&sample(), &[]));
    let expected: Vec<SessionId> = (1..=4).map(|n| SessionId(FIRST_RESTORED_ID + n)).collect();
    assert_eq!(workspace.tabs(), expected);
}

#[test]
fn sidebar_font_size_and_window_survive() {
    let mut workspace = sample();
    workspace.sidebar_open = false;
    let mut snapshot = workspace.snapshot(18.0, &|_| None);
    snapshot.window = Some(WindowSnapshot {
        width: 1200,
        height: 800,
        maximized: true,
    });
    let parsed = Snapshot::from_json(&snapshot.to_json().expect("serializes")).expect("parses");
    assert_eq!(parsed.font_size, 18.0);
    assert_eq!(parsed.window, snapshot.window);
    let (restored, _) = restore(&parsed);
    assert!(!restored.sidebar_open);
}

#[test]
fn the_sidebar_width_survives() {
    let mut workspace = sample();
    workspace.sidebar_width = Some(280.0);
    let snapshot = snapshot_of(&workspace, &[]);
    let parsed = Snapshot::from_json(&snapshot.to_json().expect("serializes")).expect("parses");
    let (restored, _) = restore(&parsed);
    assert_eq!(restored.sidebar_width, Some(280.0));
}

#[test]
fn an_unusable_sidebar_width_falls_back_to_the_default() {
    let mut snapshot = snapshot_of(&sample(), &[]);
    snapshot.sidebar_width = Some(-5.0);
    let (restored, _) = restore(&snapshot);
    assert_eq!(restored.sidebar_width, None);
}

#[test]
fn missing_window_is_left_out_of_the_file() {
    let snapshot = snapshot_of(&sample(), &[]);
    assert!(snapshot.window.is_none());
    assert!(!snapshot.to_json().expect("serializes").contains("window"));
}

#[test]
fn active_instance_index_survives() {
    let mut workspace = sample();
    workspace.active = 1;
    let (restored, _) = restore(&snapshot_of(&workspace, &[]));
    assert_eq!(restored.active, 1);
    assert_eq!(
        restored.active_instance().map(|item| item.name.as_str()),
        Some("notes")
    );
}

#[test]
fn new_instances_after_a_restore_do_not_reuse_ids() {
    let (mut restored, _) = restore(&snapshot_of(&sample(), &[]));
    restored.add_instance(None, SessionId(500));
    let mut ids: Vec<u32> = restored.instances.iter().map(|item| item.id.0).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 3);
    let mut panes: Vec<_> = restored
        .instances
        .iter()
        .flat_map(|item| item.root.panes())
        .collect();
    panes.sort();
    panes.dedup();
    assert_eq!(panes.len(), 4);
}
