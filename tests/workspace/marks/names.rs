use trame::preferences::Preferences;
use trame::workspace::{Numbering, TabSnapshot, Workspace};

use crate::snapshot::restore;
use crate::tab;

const PHRASE: &str = "Terminal {number}";

const KEEP_NUMBERS: Numbering = Numbering {
    shown: true,
    on_close: false,
    on_move: false,
};

fn with_unnamed(mut workspace: Workspace, count: u64) -> Workspace {
    for id in 1..=count {
        workspace.add_instance(None, tab(id));
    }
    workspace
}

fn shown(workspace: &Workspace) -> Vec<String> {
    workspace
        .instances
        .iter()
        .map(|instance| instance.shown_name(PHRASE).into_owned())
        .collect()
}

#[test]
fn an_unnamed_instance_is_saved_without_a_name() {
    let workspace = with_unnamed(Workspace::new(), 1);
    let snapshot = workspace.snapshot(Preferences::default(), &|_| TabSnapshot::default());
    assert_eq!(snapshot.instances[0].name, "");
}

#[test]
fn an_unnamed_instance_is_shown_with_the_phrase_it_is_given() {
    let mut workspace = with_unnamed(Workspace::new(), 1);
    workspace.add_instance(Some("build".to_owned()), tab(2));
    assert_eq!(shown(&workspace), ["Terminal 1", "build"]);
}

#[test]
fn an_unnamed_instance_follows_its_number_when_another_closes() {
    let mut workspace = with_unnamed(Workspace::new(), 3);
    workspace.remove_instance(0);
    assert_eq!(shown(&workspace), ["Terminal 1", "Terminal 2"]);
}

#[test]
fn an_unnamed_instance_keeps_its_shown_name_after_a_restart() {
    let mut workspace = with_unnamed(Workspace::numbered(KEEP_NUMBERS), 3);
    workspace.remove_instance(0);
    let snapshot = workspace.snapshot(Preferences::default(), &|_| TabSnapshot::default());
    let (restored, _) = restore(&snapshot);
    assert_eq!(shown(&workspace), ["Terminal 2", "Terminal 3"]);
    assert_eq!(shown(&restored), shown(&workspace));
}
