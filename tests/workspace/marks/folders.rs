use std::path::PathBuf;

use trame::theme::Swatch;
use trame::workspace::{InstanceId, Snapshot, Workspace};

use crate::snapshot::{restore, snapshot_of};
use crate::{tab, tabs, workspace_with};

fn first_instance(workspace: &Workspace) -> InstanceId {
    workspace.instances[0].id
}

#[test]
fn an_instance_starts_without_a_folder_and_keeps_the_one_it_gets() {
    let mut workspace = workspace_with(&[1]);
    let id = first_instance(&workspace);
    assert_eq!(workspace.instances[0].folder, None);
    workspace.set_instance_folder(id, Some(PathBuf::from("projects")));
    assert_eq!(
        workspace.instances[0].folder,
        Some(PathBuf::from("projects"))
    );
    workspace.set_instance_folder(id, None);
    assert_eq!(workspace.instances[0].folder, None);
}

#[test]
fn an_unknown_instance_gets_no_folder() {
    let mut workspace = workspace_with(&[1]);
    workspace.set_instance_folder(InstanceId(99), Some(PathBuf::from("projects")));
    assert_eq!(workspace.instances[0].folder, None);
}

#[test]
fn the_instance_folder_survives_a_save() {
    let mut workspace = workspace_with(&[1]);
    let id = first_instance(&workspace);
    workspace.set_instance_folder(id, Some(PathBuf::from("projects")));
    let text = snapshot_of(&workspace, &[]).to_json().expect("serializes");
    let (restored, _) = restore(&Snapshot::from_json(&text).expect("parses"));
    assert_eq!(
        restored.instances[0].folder,
        Some(PathBuf::from("projects"))
    );
}

#[test]
fn a_replaced_tab_keeps_its_place_its_focus_and_its_marks() {
    let mut workspace = workspace_with(&[1, 2, 3]);
    workspace.activate_tab(tab(2));
    workspace.rename_tab(tab(2), "build");
    workspace.set_tab_color(tab(2), Some(Swatch::Red));
    assert!(workspace.replace_tab(tab(2), tab(7)));
    assert_eq!(workspace.tabs(), tabs(&[1, 7, 3]));
    assert_eq!(workspace.active_tab(), Some(tab(7)));
    assert_eq!(workspace.tab_name(tab(7)), Some("build"));
    assert_eq!(workspace.tab_color(tab(7)), Some(Swatch::Red));
    assert_eq!(workspace.tab_name(tab(2)), None);
}

#[test]
fn replacing_an_unknown_tab_changes_nothing() {
    let mut workspace = workspace_with(&[1, 2]);
    let before = workspace.clone();
    assert!(!workspace.replace_tab(tab(9), tab(7)));
    assert_eq!(workspace, before);
}
