use trame::session::SessionId;
use trame::ui::Hit;
use trame::ui::frame::SidebarItem;

use crate::{frame, text_system, workspace};

#[test]
fn a_folded_instance_hides_its_terminals_and_keeps_its_count() {
    let mut text = text_system();
    let mut workspace = workspace();
    let id = workspace.instances[0].id;
    workspace.toggle_folded(id);
    let layout = frame(&mut text, &workspace);
    let rows = &layout.sidebar.as_ref().expect("the sidebar").rows;
    let shown = |tab: u64| {
        rows.iter()
            .any(|row| row.item == SidebarItem::Tab(SessionId(tab)))
    };
    assert!(!shown(1) && !shown(2) && shown(3));
    let expected = text.layout("2", layout.metrics.caption());
    let count = rows[0].count.as_ref().expect("the hidden count");
    assert_eq!(count.width, expected.width);
    assert!(rows[0].folded);
    assert_eq!(workspace.instances[0].active_tab(), Some(SessionId(2)));
}

#[test]
fn the_chevron_of_an_instance_row_toggles_folding() {
    let mut text = text_system();
    let workspace = workspace();
    let layout = frame(&mut text, &workspace);
    let sidebar = layout.sidebar.as_ref().expect("the sidebar");
    let row = &sidebar.rows[0];
    let fold = row.fold.expect("an instance row has a chevron");
    assert!(fold.right() <= row.close.x);
    let hit = layout.hit(fold.center(), false);
    assert_eq!(hit, Some(Hit::RowFold(workspace.instances[0].id)));
    assert_eq!(sidebar.rows.len(), 5);
    assert_eq!(sidebar.rows[4].item, SidebarItem::Tab(SessionId(3)));
    assert!(sidebar.rows[1].fold.is_none());
}
