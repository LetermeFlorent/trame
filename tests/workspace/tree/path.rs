use trame::workspace::Node;

use crate::{column, leaf, row};

#[test]
fn node_mut_follows_a_path_of_child_indexes() {
    let mut root = row(vec![leaf(1), column(vec![leaf(2), leaf(3)])]);
    assert_eq!(root.node_mut(&[1, 0]), Some(&mut leaf(2)));
    assert_eq!(root.node_mut(&[1, 5]), None);
    assert_eq!(root.node_mut(&[0, 0]), None);
    assert!(matches!(root.node_mut(&[]), Some(Node::Split { .. })));
}
