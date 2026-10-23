use trame::ui::Hit;
use trame::workspace::Workspace;

use crate::{frame, text_system, workspace};

pub fn closed_sidebar() -> Workspace {
    let mut workspace = workspace();
    workspace.sidebar_open = false;
    workspace
}

#[test]
fn the_settings_button_sits_at_the_bottom_of_the_side() {
    let mut text = text_system();
    let layout = frame(&mut text, &workspace());
    let footer = &layout.footer;
    assert_eq!(footer.rect.bottom(), layout.side.bottom());
    assert!(footer.rect.contains(footer.settings.center()));
    assert_eq!(
        layout.hit(footer.settings.center(), false),
        Some(Hit::Settings)
    );
    assert!(layout.settings.is_none());
}

#[test]
fn sidebar_and_rail_end_above_the_footer() {
    let mut text = text_system();
    let open = frame(&mut text, &workspace());
    let sidebar = open.sidebar.as_ref().expect("the sidebar");
    assert!(sidebar.rect.bottom() <= open.footer.rect.y);
    let collapsed = frame(&mut text, &closed_sidebar());
    let rail = collapsed.rail.as_ref().expect("the rail");
    assert!(rail.rect.bottom() <= collapsed.footer.rect.y);
    assert_eq!(
        collapsed.hit(collapsed.footer.settings.center(), false),
        Some(Hit::Settings)
    );
}

#[test]
fn the_open_sidebar_widens_the_button_to_hold_its_name() {
    let mut text = text_system();
    let open = frame(&mut text, &workspace());
    let footer = &open.footer;
    assert!(footer.settings.width > footer.icon.width);
    assert!(footer.settings.right() <= open.side.right());
    assert!(footer.label_x >= footer.icon.right());
    assert!(footer.label_x + footer.label.width <= footer.settings.right());
    let on_name = footer.settings.center();
    assert!(!footer.icon.contains(on_name));
    assert_eq!(open.hit(on_name, false), Some(Hit::Settings));
    let collapsed = frame(&mut text, &closed_sidebar());
    assert_eq!(collapsed.footer.settings, collapsed.footer.icon);
}
