use trame::geometry::Point;
use trame::ui::{Density, FrameLayout, Hit, PanelSide, UiMetrics};

use crate::{WIDTH, frame_in, text_system, workspace};

fn on(panel: PanelSide, density: Density) -> UiMetrics {
    UiMetrics::new(1.0).with_display(panel, density)
}

fn row_height(frame: &FrameLayout) -> f32 {
    frame.sidebar.as_ref().expect("the panel is open").rows[0]
        .rect
        .height
}

#[test]
fn a_right_panel_sits_after_the_terminals() {
    let mut text = text_system();
    let frame = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Right, Density::Compact),
    );
    assert_eq!(frame.side.right(), WIDTH);
    assert_eq!(frame.content.x, 0.0);
    assert!(frame.content.right() <= frame.side.x);
    let sidebar = frame.sidebar.as_ref().expect("the panel is open");
    assert_eq!(sidebar.rect.x, frame.side.x);
    let edge = Point::new(frame.side.x, frame.side.center().y);
    assert_eq!(frame.hit(edge, false), Some(Hit::SidebarEdge));
}

#[test]
fn both_sides_leave_the_terminals_the_same_room() {
    let mut text = text_system();
    let left = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Left, Density::Compact),
    );
    let right = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Right, Density::Compact),
    );
    assert_eq!(left.side.width, right.side.width);
    assert_eq!(left.content.width, right.content.width);
    assert_eq!(left.content.x, right.side.width);
}

#[test]
fn spacious_rows_and_tab_bars_are_taller() {
    let mut text = text_system();
    let compact = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Left, Density::Compact),
    );
    let spacious = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Left, Density::Spacious),
    );
    assert!(row_height(&spacious) > row_height(&compact));
    let bar = |frame: &FrameLayout| frame.panes[0].strip.rect.height;
    assert!(bar(&spacious) > bar(&compact));
}

#[test]
fn compact_keeps_the_usual_sizes() {
    let mut text = text_system();
    let frame = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Left, Density::Compact),
    );
    assert_eq!(row_height(&frame), UiMetrics::SIDEBAR_ROW);
    assert_eq!(frame.panes[0].strip.rect.height, UiMetrics::TAB_BAR);
}
