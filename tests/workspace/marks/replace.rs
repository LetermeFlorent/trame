use trame::theme::Swatch;

use crate::{tab, tabs, workspace_with};

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

#[test]
fn a_taken_tab_carries_its_marks_to_another_workspace() {
    let mut workspace = workspace_with(&[1, 2]);
    workspace.rename_tab(tab(2), "build");
    workspace.set_tab_color(tab(2), Some(Swatch::Red));
    let mark = workspace.take_tab(tab(2)).expect("the tab");
    assert_eq!(workspace.tabs(), tabs(&[1]));
    assert_eq!(workspace.tab_name(tab(2)), None);
    let mut other = workspace_with(&[5]);
    other.mark_tab(tab(5), mark);
    assert_eq!(other.tab_name(tab(5)), Some("build"));
    assert_eq!(other.tab_color(tab(5)), Some(Swatch::Red));
    assert!(workspace.take_tab(tab(9)).is_none());
}
