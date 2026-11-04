use std::path::PathBuf;

use trame::workspace::{InstanceId, Workspace};

use crate::snapshot::{restore, snapshot_of, through_json};
use crate::workspace_with;

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
    let (restored, _) = restore(&through_json(&snapshot_of(&workspace, &[])));
    assert_eq!(
        restored.instances[0].folder,
        Some(PathBuf::from("projects"))
    );
}
