mod emblems;
mod folders;

use trame::theme::Swatch;
use trame::workspace::{Edge, Instance, Workspace};

use crate::{tab, workspace_with};

#[test]
fn a_tab_has_no_name_and_no_colour_by_default() {
    let workspace = workspace_with(&[1, 2]);
    assert_eq!(workspace.tab_name(tab(1)), None);
    assert_eq!(workspace.tab_color(tab(1)), None);
}

#[test]
fn renaming_a_tab_stores_the_trimmed_name() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.rename_tab(tab(2), "  build  ");
    assert_eq!(workspace.tab_name(tab(2)), Some("build"));
    assert_eq!(workspace.tab_name(tab(1)), None);
}

#[test]
fn an_empty_name_restores_the_automatic_title() {
    let mut workspace = workspace_with(&[1]);
    workspace.rename_tab(tab(1), "build");
    workspace.rename_tab(tab(1), "");
    assert_eq!(workspace.tab_name(tab(1)), None);
    workspace.rename_tab(tab(1), "build");
    workspace.rename_tab(tab(1), "   ");
    assert_eq!(workspace.tab_name(tab(1)), None);
}

#[test]
fn a_tab_colour_is_set_then_reset() {
    let mut workspace = workspace_with(&[1]);
    workspace.set_tab_color(tab(1), Some(Swatch::Teal));
    assert_eq!(workspace.tab_color(tab(1)), Some(Swatch::Teal));
    workspace.set_tab_color(tab(1), Some(Swatch::Pink));
    assert_eq!(workspace.tab_color(tab(1)), Some(Swatch::Pink));
    workspace.set_tab_color(tab(1), None);
    assert_eq!(workspace.tab_color(tab(1)), None);
}

#[test]
fn name_and_colour_are_independent() {
    let mut workspace = workspace_with(&[1]);
    workspace.rename_tab(tab(1), "logs");
    workspace.set_tab_color(tab(1), Some(Swatch::Red));
    workspace.set_tab_color(tab(1), None);
    assert_eq!(workspace.tab_name(tab(1)), Some("logs"));
    workspace.set_tab_color(tab(1), Some(Swatch::Red));
    workspace.rename_tab(tab(1), "");
    assert_eq!(workspace.tab_color(tab(1)), Some(Swatch::Red));
}

#[test]
fn an_unknown_tab_is_left_alone() {
    let mut workspace = workspace_with(&[1]);
    let before = workspace.clone();
    workspace.rename_tab(tab(9), "ghost");
    workspace.set_tab_color(tab(9), Some(Swatch::Blue));
    assert_eq!(workspace, before);
}

#[test]
fn closing_a_tab_forgets_its_name_and_colour() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.rename_tab(tab(2), "gone");
    workspace.set_tab_color(tab(2), Some(Swatch::Green));
    let mut pristine = workspace_with(&[1]);
    pristine.instances[0].emblem = workspace.instances[0].emblem;
    workspace.close_tab(tab(2));
    assert_eq!(workspace, pristine);
}

#[test]
fn removing_an_instance_forgets_the_marks_of_its_tabs() {
    let mut workspace = workspace_with(&[1]);
    workspace.add_instance(None, tab(2));
    workspace.rename_tab(tab(2), "gone");
    workspace.remove_instance(1);
    assert_eq!(workspace.tab_name(tab(2)), None);
}

#[test]
fn marks_stay_with_their_tab_when_the_layout_changes() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.rename_tab(tab(2), "kept");
    workspace.split_active(tab(3), Edge::Right);
    assert_eq!(workspace.tab_name(tab(2)), Some("kept"));
}

#[test]
fn renaming_an_instance_changes_its_name() {
    let mut workspace = workspace_with(&[1]);
    let id = workspace.instances[0].id;
    workspace.rename_instance(id, " site ");
    assert_eq!(workspace.instances[0].name, "site");
}

#[test]
fn an_empty_instance_name_restores_the_automatic_one() {
    let mut workspace = workspace_with(&[1]);
    let id = workspace.instances[0].id;
    workspace.rename_instance(id, "site");
    workspace.rename_instance(id, "");
    assert_eq!(workspace.instances[0].name, Instance::automatic_name(id));
    assert_eq!(workspace.instances[0].name, "Instance 1");
}

#[test]
fn an_instance_colour_is_set_then_reset() {
    let mut workspace = workspace_with(&[1]);
    let id = workspace.instances[0].id;
    workspace.set_instance_color(id, Some(Swatch::Orange));
    assert_eq!(workspace.instances[0].color, Some(Swatch::Orange));
    workspace.set_instance_color(id, None);
    assert_eq!(workspace.instances[0].color, None);
}

#[test]
fn only_the_named_instance_changes() {
    let mut workspace = Workspace::new();
    workspace.add_instance(None, tab(1));
    workspace.add_instance(None, tab(2));
    let first = workspace.instances[0].id;
    workspace.set_instance_color(first, Some(Swatch::Blue));
    workspace.rename_instance(first, "one");
    assert_eq!(workspace.instances[1].color, None);
    assert_eq!(workspace.instances[1].name, "Instance 2");
}
