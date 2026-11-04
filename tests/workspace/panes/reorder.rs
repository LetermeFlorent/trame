use crate::{pane_with, tab};

#[test]
fn reorder_of_an_unknown_tab_is_a_no_op() {
    let mut pane = pane_with(&[1, 2, 3], 1);
    pane.reorder(tab(9), 0);
    assert_eq!(pane, pane_with(&[1, 2, 3], 1));
}
