use trame::geometry::Point;
use trame::preferences::{MAX_INTERFACE_SIZE, MIN_INTERFACE_SIZE, Preferences};
use trame::ui::frame::{SettingControl, SettingsFrame};
use trame::ui::metrics::{MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use crate::{frame, text_system, workspace};

fn with_settings(workspace: &Workspace, preferences: &Preferences) -> FrameLayout {
    let mut text = text_system();
    let mut layout = frame(&mut text, workspace);
    layout.place_settings(&mut text, preferences);
    layout
}

fn panel(layout: &FrameLayout) -> &SettingsFrame {
    layout.settings.as_ref().expect("the settings panel")
}

fn closed_sidebar() -> Workspace {
    let mut workspace = workspace();
    workspace.sidebar_open = false;
    workspace
}

#[test]
fn the_settings_button_sits_at_the_bottom_of_the_side() {
    let mut text = text_system();
    let layout = frame(&mut text, &workspace());
    let footer = layout.footer;
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
fn the_panel_opens_above_the_button_with_one_hit_per_control() {
    let layout = with_settings(&workspace(), &Preferences::default());
    let settings = panel(&layout);
    assert!(settings.rect.bottom() <= layout.footer.settings.y);
    assert!(settings.rect.y >= layout.bounds.y);
    assert!(settings.restores_text);
    let controls = [
        (settings.restore.rect, SettingControl::RestoreText),
        (settings.text_size.smaller, SettingControl::TextSmaller),
        (settings.text_size.bigger, SettingControl::TextBigger),
        (
            settings.interface_size.smaller,
            SettingControl::InterfaceSmaller,
        ),
        (
            settings.interface_size.bigger,
            SettingControl::InterfaceBigger,
        ),
    ];
    for (rect, control) in controls {
        assert!(settings.rect.contains(rect.center()), "{control:?}");
        assert_eq!(
            layout.hit(rect.center(), false),
            Some(Hit::Setting(control))
        );
    }
    assert_eq!(
        layout.hit(settings.caption.rect.center(), false),
        Some(Hit::SettingsPanel)
    );
}

#[test]
fn the_panel_covers_the_terminal_under_it() {
    let layout = with_settings(&closed_sidebar(), &Preferences::default());
    let settings = panel(&layout);
    assert!(settings.rect.right() > layout.content.x);
    let over_content = Point::new(layout.content.x + 4.0, settings.caption.rect.center().y);
    assert!(settings.rect.contains(over_content));
    assert_eq!(layout.hit(over_content, false), Some(Hit::SettingsPanel));
}

#[test]
fn steppers_stop_at_their_limits() {
    let smallest = Preferences {
        font_size: MIN_TERMINAL_FONT_SIZE,
        restore_text: false,
        interface_size: MIN_INTERFACE_SIZE,
        ..Preferences::default()
    };
    let layout = with_settings(&workspace(), &smallest);
    let settings = panel(&layout);
    assert!(!settings.restores_text);
    for stepper in [&settings.text_size, &settings.interface_size] {
        assert!(!stepper.can_shrink && stepper.can_grow);
    }
    let largest = Preferences {
        font_size: MAX_TERMINAL_FONT_SIZE,
        restore_text: true,
        interface_size: MAX_INTERFACE_SIZE,
        ..Preferences::default()
    };
    let layout = with_settings(&workspace(), &largest);
    let settings = panel(&layout);
    for stepper in [&settings.text_size, &settings.interface_size] {
        assert!(stepper.can_shrink && !stepper.can_grow);
    }
}
