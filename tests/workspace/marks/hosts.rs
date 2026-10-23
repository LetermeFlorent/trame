use trame::preferences::SshHost;
use trame::workspace::{NodeSnapshot, Snapshot};

use crate::snapshot::{restore, snapshot_of};
use crate::{tab, workspace_with};

fn server() -> SshHost {
    SshHost::parse("me@box:2222").expect("a valid connection")
}

#[test]
fn a_tab_keeps_its_connection_until_it_closes() {
    let mut workspace = workspace_with(&[1, 2]);
    assert_eq!(workspace.tab_host(tab(1)), None);
    workspace.set_tab_host(tab(1), Some(server()));
    assert_eq!(workspace.tab_host(tab(1)), Some(&server()));
    assert_eq!(workspace.tab_host(tab(2)), None);
    workspace.set_tab_host(tab(9), Some(server()));
    assert_eq!(workspace.tab_host(tab(9)), None);
    assert!(workspace.close_tab(tab(1)));
    assert_eq!(workspace.tab_host(tab(1)), None);
}

#[test]
fn a_replaced_tab_keeps_its_connection() {
    let mut workspace = workspace_with(&[1]);
    workspace.set_tab_host(tab(1), Some(server()));
    assert!(workspace.replace_tab(tab(1), tab(5)));
    assert_eq!(workspace.tab_host(tab(5)), Some(&server()));
}

#[test]
fn the_connection_of_a_tab_survives_a_save() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.set_tab_host(tab(2), Some(server()));
    let text = snapshot_of(&workspace, &[]).to_json().expect("serializes");
    let (restored, seen) = restore(&Snapshot::from_json(&text).expect("parses"));
    assert_eq!(seen[1].ssh, Some(server()));
    let second = restored.tabs()[1];
    assert_eq!(restored.tab_host(second), Some(&server()));
    assert_eq!(restored.tab_host(restored.tabs()[0]), None);
}

#[test]
fn a_saved_connection_that_could_run_something_else_is_not_restored() {
    let mut workspace = workspace_with(&[1]);
    workspace.set_tab_host(tab(1), Some(server()));
    let mut snapshot = snapshot_of(&workspace, &[]);
    let NodeSnapshot::Pane(pane) = &mut snapshot.instances[0].root else {
        panic!("a single pane");
    };
    pane.tabs[0].ssh = Some(SshHost {
        target: "box;calc".to_owned(),
        port: None,
    });
    let (restored, _) = restore(&snapshot);
    assert_eq!(restored.tab_host(restored.tabs()[0]), None);
}
