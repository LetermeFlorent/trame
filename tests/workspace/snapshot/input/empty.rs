use trame::session::SessionId;
use trame::workspace::{SnapshotError, Workspace};

use crate::snapshot::{pane, single, snapshot_of, through_json};

#[test]
fn empty_workspace_has_nothing_to_restore() {
    let snapshot = snapshot_of(&Workspace::new(), &[]);
    assert!(snapshot.instances.is_empty());
    let result = Workspace::restore(&snapshot, &mut |_| SessionId(1));
    assert!(matches!(result, Err(SnapshotError::Empty)));
}

#[test]
fn empty_workspace_round_trips_through_json() {
    let snapshot = snapshot_of(&Workspace::new(), &[]);
    assert_eq!(through_json(&snapshot), snapshot);
}

#[test]
fn snapshot_with_only_empty_panes_has_nothing_to_restore() {
    let result = Workspace::restore(&single(pane(0)), &mut |_| SessionId(1));
    assert!(matches!(result, Err(SnapshotError::Empty)));
}
