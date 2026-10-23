use std::path::PathBuf;

use trame::preferences::{Preferences, Recall};
use trame::workspace::{Axis, NodeSnapshot, PaneSnapshot, Snapshot, TabSnapshot};

use super::{instance_of, pane, restore, shape, single};

fn saved(preferences: Preferences) -> Snapshot {
    let busy = NodeSnapshot::Split {
        axis: Axis::Row,
        ratios: vec![1.0, 1.0],
        pinned: Vec::new(),
        children: vec![
            pane(2),
            NodeSnapshot::Pane(PaneSnapshot {
                tabs: vec![
                    TabSnapshot {
                        transcript: Some("abc".to_owned()),
                        ..TabSnapshot::default()
                    },
                    TabSnapshot::default(),
                    TabSnapshot::default(),
                ],
                active: 2,
            }),
        ],
    };
    let mut snapshot = single(busy);
    snapshot.instances[0].active_pane = 1;
    snapshot.instances.push(instance_of(pane(1)));
    snapshot.instances[1].folder = Some(PathBuf::from("projects"));
    snapshot.preferences = preferences;
    snapshot
}

#[test]
fn with_everything_remembered_the_snapshot_comes_back_whole() {
    let snapshot = saved(Preferences::default());
    assert_eq!(snapshot.recalled(), Some(snapshot.clone()));
}

#[test]
fn forgotten_instances_bring_nothing_back() {
    let off = Preferences::default().toggle(Recall::Instances);
    assert_eq!(saved(off).recalled(), None);
}

#[test]
fn forgotten_terminals_keep_the_instances_with_fresh_terminals() {
    let preferences = Preferences {
        default_folder: Some(PathBuf::from("home")),
        instance_tabs: 3,
        ..Preferences::default().toggle(Recall::Tabs)
    };
    let snapshot = saved(preferences);
    let recalled = snapshot.recalled().expect("instances come back");
    assert_eq!(recalled.instances.len(), snapshot.instances.len());
    let (workspace, seen) = restore(&recalled);
    for (instance, saved) in workspace.instances.iter().zip(&snapshot.instances) {
        assert_eq!(instance.name, saved.name);
        assert_eq!(instance.folder, saved.folder);
        assert_eq!(shape(instance), "[3]");
    }
    let folders: Vec<_> = seen.iter().map(|tab| tab.cwd.clone()).collect();
    let home = Some(PathBuf::from("home"));
    let projects = Some(PathBuf::from("projects"));
    assert_eq!(
        folders,
        [
            home.clone(),
            home.clone(),
            home,
            projects.clone(),
            projects.clone(),
            projects
        ]
    );
    assert!(seen.iter().all(|tab| tab.transcript.is_none()));
}
