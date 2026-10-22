use trame::geometry::Point;
use trame::session::SessionId;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::ui::drop::DropMark;
use trame::ui::frame::SidebarItem;
use trame::ui::{FrameInput, FrameLayout, UiMetrics};
use trame::workspace::{DropTarget, Edge, InstanceId, Workspace};

const WIDTH: f32 = 1200.0;
const HEIGHT: f32 = 700.0;
const FONT_SIZE: f32 = 14.0;

fn workspace() -> Workspace {
    let mut workspace = Workspace::new();
    workspace.add_instance(None, SessionId(1));
    workspace.add_tab(SessionId(2));
    workspace.add_tab(SessionId(3));
    workspace.add_instance(None, SessionId(4));
    workspace.active = 0;
    workspace
}

fn frame(workspace: &Workspace) -> FrameLayout {
    let mut text =
        TextSystem::new(FONT_SIZE, DEFAULT_LINE_HEIGHT).expect("a monospace font is installed");
    let title = |id: SessionId| format!("tab {}", id.0);
    let input = FrameInput {
        workspace,
        width: WIDTH,
        height: HEIGHT,
        metrics: UiMetrics::new(1.0),
        maximized: false,
        title: &title,
    };
    FrameLayout::compute(&mut text, &input)
}

fn instance(workspace: &Workspace) -> InstanceId {
    workspace.active_instance().expect("an instance").id
}

#[test]
fn the_tab_bar_gives_an_insertion_index_and_a_marker() {
    let workspace = workspace();
    let frame = frame(&workspace);
    let pane = &frame.panes[0];
    let second = pane.strip.tabs[1].rect;
    let point = Point::new(second.x + 2.0, second.center().y);
    let spot = frame.drop_at(point, instance(&workspace)).expect("a spot");
    let expected = DropTarget::Tabs {
        instance: instance(&workspace),
        pane: pane.id,
        index: Some(1),
    };
    assert_eq!(spot.target, expected);
    let DropMark::Marker(marker) = spot.mark else {
        panic!("a tab bar drop shows a marker");
    };
    assert!((marker.center().x - second.x).abs() <= 1.0);
}

#[test]
fn past_the_last_tab_inserts_at_the_end() {
    let workspace = workspace();
    let frame = frame(&workspace);
    let last = frame.panes[0].strip.tabs.last().expect("tabs").rect;
    let point = Point::new(last.right() + 20.0, last.center().y);
    let spot = frame.drop_at(point, instance(&workspace)).expect("a spot");
    assert!(matches!(
        spot.target,
        DropTarget::Tabs { index: Some(3), .. }
    ));
}

#[test]
fn the_pane_body_splits_on_an_edge_or_adds_in_the_middle() {
    let workspace = workspace();
    let frame = frame(&workspace);
    let body = frame.panes[0].body;
    let id = instance(&workspace);
    let middle = frame.drop_at(body.center(), id).expect("a spot");
    assert!(matches!(
        middle.target,
        DropTarget::Tabs { index: None, .. }
    ));
    assert_eq!(middle.mark, DropMark::Area(body));
    let right = Point::new(body.right() - 5.0, body.center().y);
    let spot = frame.drop_at(right, id).expect("a spot");
    assert!(matches!(
        spot.target,
        DropTarget::Split {
            edge: Edge::Right,
            ..
        }
    ));
}

#[test]
fn sidebar_rows_target_their_instance() {
    let workspace = workspace();
    let frame = frame(&workspace);
    let sidebar = frame.sidebar.as_ref().expect("the sidebar is open");
    let other = workspace.instances[1].id;
    let target_of = |item: SidebarItem| {
        let row = sidebar.rows.iter().find(|row| row.item == item)?;
        frame
            .drop_at(row.rect.center(), instance(&workspace))
            .map(|spot| spot.target)
    };
    let other_row = target_of(SidebarItem::Instance(other));
    assert_eq!(other_row, Some(DropTarget::Instance(other)));
    let own_tab = target_of(SidebarItem::Tab(SessionId(2)));
    assert_eq!(own_tab, Some(DropTarget::Instance(instance(&workspace))));
    let add = frame.drop_at(sidebar.new_instance.center(), instance(&workspace));
    assert_eq!(add.map(|spot| spot.target), Some(DropTarget::NewInstance));
}

#[test]
fn the_title_bar_is_not_a_drop_spot() {
    let workspace = workspace();
    let frame = frame(&workspace);
    let point = frame.title_bar.rect.center();
    assert_eq!(frame.drop_at(point, instance(&workspace)), None);
}

#[test]
fn a_closed_sidebar_leaves_a_rail_with_a_button_that_reopens_it() {
    let mut workspace = workspace();
    workspace.sidebar_open = false;
    let frame = frame(&workspace);
    let rail = frame.rail.expect("the rail is shown");
    assert!(frame.sidebar.is_none());
    assert_eq!(rail.rect.width, UiMetrics::RAIL_WIDTH);
    assert_eq!(frame.content.x, rail.rect.right());
    let hit = frame.hit(rail.expand.center(), false);
    assert_eq!(hit, Some(trame::ui::Hit::SidebarExpand));
    let below = Point::new(rail.rect.center().x, rail.rect.bottom() - 4.0);
    assert_eq!(frame.hit(below, false), Some(trame::ui::Hit::Sidebar));
    let open = self::frame(&self::workspace());
    assert!(open.rail.is_none());
}
