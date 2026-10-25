use trame::theme::{Emblem, Letters};
use trame::workspace::{Numbering, Workspace};

use crate::{tab, workspace_with};

fn four_instances(numbering: Numbering) -> Workspace {
    let mut workspace = workspace_with(&[1]);
    workspace.set_numbering(numbering);
    for id in 2..=4 {
        workspace.add_instance(None, tab(id));
    }
    workspace
}

fn numbers(workspace: &Workspace) -> Vec<u32> {
    workspace.instances.iter().map(|item| item.number).collect()
}

#[test]
fn new_instances_count_up_and_show_their_number() {
    let workspace = four_instances(Numbering::default());
    assert_eq!(numbers(&workspace), [1, 2, 3, 4]);
    let shown = workspace.shown_emblem(&workspace.instances[2]);
    assert_eq!(shown, Emblem::Letters(Letters::parse("3").expect("digit")));
}

#[test]
fn hidden_numbers_leave_the_icon() {
    let numbering = Numbering {
        shown: false,
        ..Numbering::default()
    };
    let workspace = four_instances(numbering);
    let instance = &workspace.instances[1];
    assert_eq!(workspace.shown_emblem(instance), instance.emblem);
}

#[test]
fn closing_renumbers_only_when_asked() {
    let mut workspace = four_instances(Numbering::default());
    workspace.remove_instance(1);
    assert_eq!(numbers(&workspace), [1, 2, 3]);
    let fixed = Numbering {
        on_close: false,
        ..Numbering::default()
    };
    let mut workspace = four_instances(fixed);
    workspace.remove_instance(1);
    assert_eq!(numbers(&workspace), [1, 3, 4]);
    workspace.add_instance(None, tab(9));
    assert_eq!(numbers(&workspace), [1, 3, 4, 5]);
}

#[test]
fn moving_keeps_numbers_unless_asked() {
    let mut workspace = four_instances(Numbering::default());
    workspace.active = 0;
    assert!(workspace.move_instance(0, 2));
    workspace.renumber_after_move();
    assert_eq!(numbers(&workspace), [2, 3, 1, 4]);
    assert_eq!(workspace.active, 2);
    let renumbered = Numbering {
        on_move: true,
        ..Numbering::default()
    };
    let mut workspace = four_instances(renumbered);
    assert!(workspace.move_instance(3, 0));
    workspace.renumber_after_move();
    assert_eq!(numbers(&workspace), [1, 2, 3, 4]);
    assert!(!workspace.move_instance(0, 9));
}
