use std::collections::HashMap;

use trame::workspace::Edge;

use crate::{tab, with_instances, workspace_with};

#[test]
fn an_instance_moves_to_another_window_with_its_panes_tabs_and_names() {
    let mut source = with_instances(workspace_with(&[1]), [2]);
    source.add_tab(tab(3));
    source.split_active(tab(4), Edge::Right);
    source.rename_tab(tab(3), "build");
    let id = source.instances[1].id;
    let moving = source.take_instance(id).expect("the instance");
    assert_eq!(source.instances.len(), 1);
    assert_eq!(source.tab_name(tab(3)), None);
    let mut target = with_instances(workspace_with(&[1]), [2]);
    let renamed = HashMap::from([(tab(2), tab(12)), (tab(3), tab(13)), (tab(4), tab(14))]);
    target.adopt_instance(moving, &renamed, 1);
    assert_eq!(target.instances.len(), 3);
    assert_eq!(target.active, 1);
    let moved = &target.instances[1];
    assert_eq!(moved.tabs(), vec![tab(12), tab(13), tab(14)]);
    assert_eq!(moved.root.panes().len(), 2);
    assert!(
        moved
            .root
            .panes()
            .iter()
            .all(|pane| moved.panes.contains_key(pane))
    );
    assert!(moved.panes.contains_key(&moved.active_pane));
    let others = [&target.instances[0], &target.instances[2]];
    assert!(others.iter().all(|other| other.id != moved.id));
    assert!(others.iter().all(|other| {
        other
            .panes
            .keys()
            .all(|pane| !moved.panes.contains_key(pane))
    }));
    assert_eq!(target.tab_name(tab(13)), Some("build"));
}
