use trame::ui::{Hit, Platform, UiMetrics};
use trame::workspace::Workspace;

use crate::{frame, frame_in, text_system, workspace};

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
    let shown = layout.side.bottom() - layout.metrics.hidden_bottom(false);
    assert_eq!(footer.rect.bottom(), shown);
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

fn on(platform: Platform, scale: f32) -> UiMetrics {
    UiMetrics {
        platform,
        ..UiMetrics::new(scale)
    }
}

#[test]
fn the_settings_button_has_as_much_visible_room_above_as_below() {
    let mut text = text_system();
    for scale in [1.0, 1.1, 1.25, 1.375, 1.5, 1.75, 2.0] {
        for border in [0.0, 1.0] {
            let metrics = on(Platform::Windows, scale).with_frame_border(border);
            let layout = frame_in(&mut text, &workspace(), metrics);
            let footer = &layout.footer;
            let visible = layout.side.bottom() - metrics.hidden_bottom(false);
            let above = footer.settings.y - footer.rect.y - metrics.line();
            let below = visible - footer.settings.bottom();
            assert_eq!(above, below, "scale {scale} border {border}");
            assert!(above > 0.0, "scale {scale} border {border}");
        }
    }
}

#[test]
fn an_unmaximized_windows_window_hides_its_last_row_and_its_border() {
    let windows = on(Platform::Windows, 1.0);
    let bordered = windows.with_frame_border(1.0);
    assert_eq!(windows.hidden_bottom(false), 1.0);
    assert_eq!(bordered.hidden_bottom(false), 2.0);
    assert_eq!(bordered.hidden_bottom(true), 0.0);
    for platform in [Platform::Mac, Platform::Linux] {
        let other = UiMetrics {
            platform,
            ..bordered
        };
        assert_eq!(other.hidden_bottom(false), 0.0);
    }
}
