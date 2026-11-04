mod fold;
mod footer;
mod names;
mod rail;
mod scroll;
mod settings;
mod side;
mod update;

use trame::geometry::Point;
use trame::session::SessionId;
use trame::ui::frame::{dragged_width, shown_width};
use trame::ui::{FrameLayout, Hit, UiMetrics};
use trame::workspace::{Edge, MIN_PANE};

use crate::{frame, input, text_system, workspace};

const NARROW_WINDOW: f32 = 520.0;

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
fn panes_keep_a_minimum_width_in_a_narrow_window() {
    let mut text = text_system();
    let mut workspace = workspace();
    workspace.sidebar_width = Some(UiMetrics::SIDEBAR_MAX_WIDTH);
    workspace.split_active(SessionId(9), Edge::Right);
    let mut input = input(&workspace, UiMetrics::new(1.0));
    input.width = NARROW_WINDOW;
    let layout = FrameLayout::compute(&mut text, &input);
    assert_eq!(layout.panes.len(), 2);
    for pane in &layout.panes {
        assert!(
            pane.rect.width >= MIN_PANE,
            "a pane is {} wide",
            pane.rect.width
        );
    }
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
