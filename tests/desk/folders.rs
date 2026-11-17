use std::fs;

use trame::desk::{Effect, FolderTarget, Grids, Placement};

use crate::sandbox::Sandbox;
use crate::support::desk;
use crate::tabs::opened;

#[test]
fn a_picked_folder_relaunches_the_tab_there() {
    let sandbox = Sandbox::new("desk-relaunch");
    let (mut desk, old) = opened();
    let effects = desk.folder_picked(FolderTarget::Tab(old), sandbox.root.clone());
    assert_eq!(
        effects.as_slice(),
        [Effect::Relayout, Effect::StartQueued, Effect::Changed]
    );
    let fresh = desk.workspace.active_tab().expect("the relaunched tab");
    assert_ne!(fresh, old);
    assert!(desk.sessions.get(old).is_none());
    desk.start_queued(&Grids::default());
    let cwd = desk.sessions.cwd(fresh).expect("a folder");
    let wanted = fs::canonicalize(&sandbox.root).expect("the sandbox");
    assert_eq!(fs::canonicalize(cwd).expect("the folder"), wanted);
}

#[test]
fn folders_come_from_preferences_and_instances() {
    let sandbox = Sandbox::new("desk-folders");
    let mut desk = desk();
    assert_eq!(desk.folder_of(FolderTarget::Default), None);
    assert_eq!(desk.start_folder(Placement::Instance), None);
    let next = desk
        .preferences
        .with(|next| next.default_folder = Some(sandbox.root.clone()));
    let _ = desk.store_preferences(next);
    assert_eq!(
        desk.folder_of(FolderTarget::Default),
        Some(sandbox.root.clone())
    );
    assert_eq!(
        desk.start_folder(Placement::Instance),
        Some(sandbox.root.clone())
    );
    let _ = desk.open(Placement::Instance);
    let instance = desk.workspace.active_instance().expect("an instance").id;
    desk.workspace
        .set_instance_folder(instance, Some(sandbox.root.clone()));
    let wanted = Some(sandbox.root.clone());
    assert_eq!(desk.folder_of(FolderTarget::Instance(instance)), wanted);
    assert_eq!(desk.start_folder(Placement::Tab), wanted);
}
