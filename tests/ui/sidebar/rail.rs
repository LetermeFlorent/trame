use trame::geometry::Point;
use trame::locale::Language;
use trame::theme::Swatch;
use trame::ui::frame::SidebarItem;
use trame::ui::{Hit, UiMetrics};

use crate::{frame, text_system, workspace};

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
