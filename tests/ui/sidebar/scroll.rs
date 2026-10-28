use trame::geometry::Point;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::ui::frame::BarPart;
use trame::ui::{FrameLayout, Hit};
use trame::workspace::{InstanceId, Workspace};

use crate::{frame, text_system, workspace};

const INSTANCES: u64 = 40;

fn crowded() -> Workspace {
    let mut workspace = workspace();
    for id in 0..INSTANCES {
        workspace.add_instance(None, SessionId(100 + id));
    }
    workspace
}

fn scrolled_to(text: &mut TextSystem, mut workspace: Workspace, first: usize) -> FrameLayout {
    workspace.sidebar_first = first;
    frame(text, &workspace)
}

#[test]
fn the_last_sidebar_row_can_be_scrolled_into_view() {
    let mut text = text_system();
    let layout = scrolled_to(&mut text, crowded(), usize::MAX);
    let sidebar = layout.sidebar.as_ref().expect("sidebar is open");
    let last = sidebar.rows.last().expect("the sidebar has rows");
    assert!(last.rect.y >= sidebar.list.y);
    assert!(last.rect.bottom() <= sidebar.list.bottom());
    assert_eq!(
        layout.hit(last.rect.center(), false),
        Some(Hit::Row(last.item))
    );
}

#[test]
fn a_row_scrolled_above_the_list_cannot_be_clicked() {
    let mut text = text_system();
    let layout = scrolled_to(&mut text, crowded(), 1);
    let sidebar = layout.sidebar.as_ref().expect("sidebar is open");
    let hidden = &sidebar.rows[0];
    assert!(hidden.rect.bottom() <= sidebar.list.y);
    assert_ne!(
        layout.hit(hidden.rect.center(), false),
        Some(Hit::Row(hidden.item))
    );
}

#[test]
fn a_tab_dropped_on_a_row_scrolled_away_lands_nowhere() {
    let mut text = text_system();
    let layout = scrolled_to(&mut text, crowded(), 1);
    let sidebar = layout.sidebar.as_ref().expect("sidebar is open");
    let hidden = &sidebar.rows[0];
    assert!(hidden.rect.bottom() <= sidebar.list.y);
    assert_eq!(layout.drop_at(hidden.rect.center(), InstanceId(1)), None);
}

#[test]
fn the_sidebar_shows_a_bar_only_when_its_rows_overflow() {
    let mut text = text_system();
    let roomy = scrolled_to(&mut text, workspace(), 0);
    assert!(roomy.sidebar.expect("sidebar is open").bar.is_none());
    let layout = scrolled_to(&mut text, crowded(), 0);
    let sidebar = layout.sidebar.as_ref().expect("sidebar is open");
    let bar = sidebar.bar.expect("the rows overflow");
    let below_thumb = Point::new(bar.track.center().x, bar.track.bottom() - 1.0);
    assert_eq!(
        layout.hit(bar.thumb.center(), false),
        Some(Hit::SidebarBar(BarPart::Thumb))
    );
    assert_eq!(
        layout.hit(below_thumb, false),
        Some(Hit::SidebarBar(BarPart::Track))
    );
}
