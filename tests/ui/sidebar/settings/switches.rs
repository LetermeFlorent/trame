use trame::geometry::Rect;
use trame::preferences::{MAX_INSTANCE_TABS, Preferences};
use trame::ui::frame::{SettingControl, SettingsTab, ToggleRow};
use trame::ui::{FrameLayout, Hit};

use super::terminals::stepper;
use super::{on_tab, page};

fn switch(layout: &FrameLayout, control: SettingControl) -> &ToggleRow {
    page(layout).toggle(control).expect("the switch")
}

#[track_caller]
fn assert_switch(control: SettingControl, above: impl Fn(&FrameLayout) -> Rect, off: Preferences) {
    let layout = on_tab(&Preferences::default(), SettingsTab::Terminals);
    let row = switch(&layout, control);
    assert!(row.on && row.enabled);
    assert!(above(&layout).bottom() <= row.line.rect.y);
    assert_eq!(
        layout.hit(row.line.rect.center(), false),
        Some(Hit::Setting(control))
    );
    assert!(!switch(&on_tab(&off, SettingsTab::Terminals), control).on);
}

#[test]
fn the_terminal_count_stepper_stops_at_its_limits() {
    let fewer = SettingControl::InstanceTabsFewer;
    let layout = on_tab(&Preferences::default(), SettingsTab::Terminals);
    let least = stepper(&layout, fewer);
    assert!(!least.can_shrink && least.can_grow);
    let most = Preferences::default().step_instance_tabs(i32::from(MAX_INSTANCE_TABS));
    let layout = on_tab(&most, SettingsTab::Terminals);
    let most = stepper(&layout, fewer);
    assert!(most.can_shrink && !most.can_grow);
}

#[test]
fn the_close_switch_sits_below_the_terminal_count_and_follows_its_preference() {
    assert_switch(
        SettingControl::ConfirmClose,
        |layout| stepper(layout, SettingControl::InstanceTabsFewer).line.rect,
        Preferences::default().toggle_confirm_close(),
    );
}

#[test]
fn the_paste_switch_sits_below_the_close_switch_and_follows_its_preference() {
    assert_switch(
        SettingControl::ConfirmPaste,
        |layout| switch(layout, SettingControl::ConfirmClose).line.rect,
        Preferences::default().with(|next| next.confirm_paste = false),
    );
}

#[test]
fn the_link_switch_sits_below_the_paste_switch_and_follows_its_preference() {
    assert_switch(
        SettingControl::DetectLinks,
        |layout| switch(layout, SettingControl::ConfirmPaste).line.rect,
        Preferences::default().with(|next| next.detect_links = false),
    );
}
