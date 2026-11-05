use trame::geometry::Point;
use trame::locale::Language;
use trame::theme::Swatch;
use trame::ui::frame::SidebarItem;
use trame::ui::{Density, Hit, PanelSide, SidebarMode, UiMetrics};

use crate::{WIDTH, frame, frame_with_metrics, text_system, workspace};

#[test]
fn a_closed_sidebar_leaves_a_rail_with_one_button_per_instance() {
    let mut text = text_system();
    let mut workspace = workspace();
    workspace.set_instance_color(workspace.instances[1].id, Some(Swatch::Teal));
    workspace.sidebar = SidebarMode::Rail;
    let layout = frame(&mut text, &workspace);
    let rail = layout.rail.as_ref().expect("the rail is shown");
    assert!(layout.sidebar.is_none());
    assert_eq!(rail.rect.width, UiMetrics::RAIL_WIDTH);
    assert_eq!(layout.content.x, rail.rect.right());
    assert_eq!(rail.slots.len(), workspace.instances.len());
    let english = Language::English.strings();
    for (slot, instance) in rail.slots.iter().zip(&workspace.instances) {
        let hit = layout.hit(slot.rect.center(), false);
        assert_eq!(hit, Some(Hit::RailInstance(instance.id)));
        assert_eq!(slot.color, instance.color);
        assert_eq!(slot.emblem, workspace.shown_emblem(instance));
        assert_eq!(slot.name, instance.shown_name(english.instance_name));
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

#[test]
fn the_toggle_cycles_open_rail_hidden_and_back() {
    let order = [SidebarMode::Open, SidebarMode::Rail, SidebarMode::Hidden];
    for (index, mode) in order.iter().enumerate() {
        assert_eq!(mode.next(), order[(index + 1) % order.len()]);
    }
    assert_eq!(SidebarMode::default(), SidebarMode::Open);
}

#[test]
fn a_hidden_panel_takes_no_room_and_keeps_its_toggle() {
    let mut text = text_system();
    let mut workspace = workspace();
    workspace.sidebar = SidebarMode::Hidden;
    for panel in PanelSide::ALL {
        let metrics = UiMetrics::new(1.0).with_display(panel, Density::Compact);
        let layout = frame_with_metrics(&mut text, &workspace, metrics);
        assert_eq!(layout.side.width, 0.0);
        assert!(layout.rail.is_none() && layout.sidebar.is_none());
        assert_eq!(layout.content.width, WIDTH);
        let x = layout.side.x.min(WIDTH - 1.0);
        for y in [layout.content.center().y, layout.footer.rect.center().y] {
            let hit = layout.hit(Point::new(x, y), false);
            assert!(matches!(hit, Some(Hit::Terminal(_))), "{hit:?}");
        }
        let toggle = layout.title_bar.toggle.center();
        assert_eq!(layout.hit(toggle, false), Some(Hit::SidebarToggle));
    }
}
