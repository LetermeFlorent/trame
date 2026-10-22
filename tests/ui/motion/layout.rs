use trame::session::SessionId;
use trame::ui::motion::DURATION;
use trame::ui::{FrameLayout, Motion, UiMetrics};
use trame::workspace::Edge;

use super::{layout_at, settle};
use crate::{frame, text_system, workspace};

const NEW_TAB: SessionId = SessionId(9);

#[test]
fn the_sidebar_slides_and_the_panes_follow() {
    let mut text = text_system();
    let mut workspace = workspace();
    let (mut motion, start) = settle(&mut text, &workspace);
    workspace.sidebar_open = false;
    motion.animate_next();
    let first = layout_at(&mut text, &workspace, &mut motion, start);
    assert_eq!(first.reveal, 1.0);
    let middle = layout_at(&mut text, &workspace, &mut motion, start + DURATION / 2);
    assert!(
        middle.reveal > 0.0 && middle.reveal < 1.0,
        "{}",
        middle.reveal
    );
    assert!(middle.side.width > UiMetrics::RAIL_WIDTH);
    assert!(middle.side.width < UiMetrics::SIDEBAR_WIDTH);
    assert!(middle.sidebar.is_some() && middle.rail.is_some());
    assert!(middle.panes[0].rect.x > UiMetrics::RAIL_WIDTH);
    assert!(motion.is_busy());
    let end = layout_at(&mut text, &workspace, &mut motion, start + DURATION);
    assert_eq!(end.reveal, 0.0);
    assert!(end.sidebar.is_none());
    assert_eq!(end.panes[0].rect.x, UiMetrics::RAIL_WIDTH);
}

#[test]
fn a_new_split_grows_from_the_edge() {
    let mut text = text_system();
    let mut workspace = workspace();
    let (mut motion, start) = settle(&mut text, &workspace);
    workspace.split_active(NEW_TAB, Edge::Right);
    motion.animate_next();
    let first = layout_at(&mut text, &workspace, &mut motion, start);
    let target = frame(&mut text, &workspace);
    let new = target.panes[1].id;
    let entering = first.pane(new).expect("new pane is laid out");
    assert_eq!(entering.rect.width, 0.0);
    assert_eq!(entering.rect.x, target.content.right());
    let middle = layout_at(&mut text, &workspace, &mut motion, start + DURATION / 2);
    let growing = middle.pane(new).expect("new pane is laid out").rect.width;
    let full = target.pane(new).expect("new pane is laid out").rect.width;
    assert!(growing > 0.0 && growing < full, "{growing}");
    let end = layout_at(&mut text, &workspace, &mut motion, start + DURATION);
    let settled = |frame: &FrameLayout| frame.pane(new).map(|pane| pane.rect);
    assert_eq!(settled(&end), settled(&target));
}

#[test]
fn the_terminal_size_follows_the_target_not_the_animation() {
    let mut text = text_system();
    let mut workspace = workspace();
    let (mut motion, start) = settle(&mut text, &workspace);
    workspace.split_active(NEW_TAB, Edge::Right);
    motion.animate_next();
    layout_at(&mut text, &workspace, &mut motion, start);
    let middle = layout_at(&mut text, &workspace, &mut motion, start + DURATION / 2);
    let target = frame(&mut text, &workspace);
    for (shown, settled) in middle.panes.iter().zip(&target.panes) {
        assert_eq!(shown.grid_area, settled.grid_area);
    }
}

#[test]
fn a_new_tab_grows_from_nothing() {
    let mut text = text_system();
    let mut workspace = workspace();
    let (mut motion, start) = settle(&mut text, &workspace);
    workspace.add_tab(NEW_TAB);
    motion.animate_next();
    let width_of = |frame: &FrameLayout| {
        let tabs = &frame.panes[0].strip.tabs;
        let tab = tabs.iter().find(|tab| tab.id == NEW_TAB)?;
        Some(tab.rect.width)
    };
    let first = layout_at(&mut text, &workspace, &mut motion, start);
    assert_eq!(width_of(&first), Some(0.0));
    let middle = layout_at(&mut text, &workspace, &mut motion, start + DURATION / 2);
    let target = frame(&mut text, &workspace);
    let full = width_of(&target).expect("new tab is laid out");
    let growing = width_of(&middle).expect("new tab is laid out");
    assert!(growing > 0.0 && growing < full, "{growing}");
}

#[test]
fn dividers_stay_on_the_shown_panes() {
    let mut text = text_system();
    let mut workspace = workspace();
    let (mut motion, start) = settle(&mut text, &workspace);
    workspace.split_active(NEW_TAB, Edge::Right);
    motion.animate_next();
    layout_at(&mut text, &workspace, &mut motion, start);
    let middle = layout_at(&mut text, &workspace, &mut motion, start + DURATION / 2);
    let divider = middle.dividers[0].rect;
    let left = middle.panes[0].rect;
    assert!(divider.x >= left.right() - divider.width && divider.x <= left.right());
}

#[test]
fn without_animation_every_change_snaps() {
    let mut text = text_system();
    let mut workspace = workspace();
    let mut motion = Motion::default();
    let now = motion.now();
    layout_at(&mut text, &workspace, &mut motion, now);
    workspace.sidebar_open = false;
    let snapped = layout_at(&mut text, &workspace, &mut motion, now);
    assert_eq!(snapped.reveal, 0.0);
    assert!(!motion.is_busy());
}
