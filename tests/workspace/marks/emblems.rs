use std::collections::HashSet;

use trame::theme::Emblem;
use trame::workspace::Workspace;

use crate::{tab, workspace_with};

#[test]
fn every_new_instance_gets_an_emblem_nobody_else_has_until_all_are_used() {
    let mut workspace = Workspace::new();
    for id in 0..Emblem::ALL.len() as u64 {
        workspace.add_instance(None, tab(id + 1));
    }
    let emblems: HashSet<Emblem> = workspace.emblems().into_iter().collect();
    assert_eq!(emblems.len(), Emblem::ALL.len());
    workspace.add_instance(None, tab(99));
    assert_eq!(workspace.instances.len(), Emblem::ALL.len() + 1);
}

#[test]
fn an_instance_emblem_can_be_changed() {
    let mut workspace = workspace_with(&[1]);
    let id = workspace.instances[0].id;
    workspace.set_instance_emblem(id, Emblem::Database);
    assert_eq!(workspace.instances[0].emblem, Emblem::Database);
    assert_eq!(
        workspace.instance(id).map(|instance| instance.emblem),
        Some(Emblem::Database)
    );
}

#[test]
fn changing_the_emblem_of_an_unknown_instance_changes_nothing() {
    let mut workspace = workspace_with(&[1]);
    let before = workspace.clone();
    let unknown = trame::workspace::InstanceId(42);
    workspace.set_instance_emblem(unknown, Emblem::Gear);
    assert_eq!(workspace, before);
    assert!(workspace.instance(unknown).is_none());
}
