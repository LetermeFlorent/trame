mod footer;
mod scroll;
mod settings;
mod side;
mod update;

use trame::geometry::Point;
use trame::theme::Swatch;
use trame::ui::frame::{SidebarItem, dragged_width, shown_width};
use trame::ui::{FrameLayout, Hit, UiMetrics};

use crate::{frame, text_system, workspace};

fn middle_of_edge(layout: &FrameLayout) -> Point {
    Point::new(
        layout.side.right(),
        layout.side.y + layout.side.height / 2.0,
    )
}

#[test]
fn the_sidebar_starts_at_its_default_width() {
    let mut text = text_system();
    let layout = frame(&mut text, &workspace());
    assert_eq!(layout.side.width, UiMetrics::SIDEBAR_WIDTH);
    assert_eq!(layout.content.x, UiMetrics::SIDEBAR_WIDTH);
}

#[test]
fn a_stored_width_moves_the_content() {
    let mut text = text_system();
    let mut workspace = workspace();
    workspace.sidebar_width = Some(300.0);
    let layout = frame(&mut text, &workspace);
    assert_eq!(layout.side.width, 300.0);
    assert_eq!(layout.content.x, 300.0);
    assert_eq!(layout.sidebar.expect("sidebar is open").rect.width, 300.0);
}

#[test]
fn the_width_stays_between_its_bounds() {
    assert_eq!(shown_width(None), UiMetrics::SIDEBAR_WIDTH);
    assert_eq!(shown_width(Some(20.0)), UiMetrics::SIDEBAR_MIN_WIDTH);
    assert_eq!(shown_width(Some(5000.0)), UiMetrics::SIDEBAR_MAX_WIDTH);
}

#[test]
fn dragging_below_the_threshold_collapses_the_sidebar() {
    let threshold = UiMetrics::SIDEBAR_COLLAPSE_WIDTH;
    assert_eq!(dragged_width(threshold - 1.0), None);
    assert_eq!(dragged_width(0.0), None);
    assert_eq!(dragged_width(threshold), Some(UiMetrics::SIDEBAR_MIN_WIDTH));
    assert_eq!(dragged_width(250.0), Some(250.0));
    assert_eq!(dragged_width(9000.0), Some(UiMetrics::SIDEBAR_MAX_WIDTH));
}

#[test]
fn the_edge_can_be_grabbed_open_or_collapsed() {
    let mut text = text_system();
    let mut workspace = workspace();
    let open = frame(&mut text, &workspace);
    assert_eq!(
        open.hit(middle_of_edge(&open), false),
        Some(Hit::SidebarEdge)
    );
    let inside = Point::new(open.side.right() - 20.0, middle_of_edge(&open).y);
    assert_eq!(open.hit(inside, false), Some(Hit::Sidebar));
    workspace.sidebar_open = false;
    let collapsed = frame(&mut text, &workspace);
    assert_eq!(collapsed.side.width, UiMetrics::RAIL_WIDTH);
    assert_eq!(
        collapsed.hit(middle_of_edge(&collapsed), false),
        Some(Hit::SidebarEdge)
    );
}

#[test]
fn a_closed_sidebar_leaves_a_rail_with_one_button_per_instance() {
    let mut text = text_system();
    let mut workspace = workspace();
    workspace.set_instance_color(workspace.instances[1].id, Some(Swatch::Teal));
    workspace.sidebar_open = false;
    let layout = frame(&mut text, &workspace);
    let rail = layout.rail.as_ref().expect("the rail is shown");
    assert!(layout.sidebar.is_none());
    assert_eq!(rail.rect.width, UiMetrics::RAIL_WIDTH);
    assert_eq!(layout.content.x, rail.rect.right());
    assert_eq!(rail.slots.len(), workspace.instances.len());
    for (slot, instance) in rail.slots.iter().zip(&workspace.instances) {
        let hit = layout.hit(slot.rect.center(), false);
        assert_eq!(hit, Some(Hit::RailInstance(instance.id)));
        assert_eq!(slot.color, instance.color);
        assert_eq!(slot.emblem, workspace.shown_emblem(instance));
        assert_eq!(slot.name, instance.name);
        assert!(rail.rect.contains(slot.icon.center()));
    }
    assert!(rail.slots[workspace.active].active);
    assert!(!rail.slots[1].active);
    let below = Point::new(rail.rect.center().x, rail.rect.bottom() - 4.0);
    assert_eq!(layout.hit(below, false), Some(Hit::Sidebar));
}

#[test]
fn an_open_sidebar_has_no_rail() {
    let mut text = text_system();
    assert!(frame(&mut text, &workspace()).rail.is_none());
}

#[test]
fn instance_rows_carry_their_emblem_and_tab_rows_none() {
    let mut text = text_system();
    let workspace = workspace();
    let layout = frame(&mut text, &workspace);
    let sidebar = layout.sidebar.as_ref().expect("the sidebar");
    for row in &sidebar.rows {
        match row.item {
            SidebarItem::Instance(id) => {
                let instance = workspace.instance(id).expect("the instance");
                assert_eq!(row.emblem, Some(workspace.shown_emblem(instance)));
            }
            SidebarItem::Tab(_) => assert_eq!(row.emblem, None),
        }
    }
}
