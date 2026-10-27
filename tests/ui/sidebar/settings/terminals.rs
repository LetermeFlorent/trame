use trame::input::TextField;
use trame::locale::{Language, Strings};
use trame::preferences::{MAX_INSTANCE_TABS, Preferences};
use trame::text::TextSystem;
use trame::ui::frame::{ChoiceRow, SettingControl, SettingsTab, Stepper, ToggleRow};
use trame::ui::{FrameLayout, Hit};

use super::{on_tab, page, placed, request_on};
use crate::text_system;

fn terminals(
    text: &mut TextSystem,
    preferences: &Preferences,
    strings: &'static Strings,
) -> FrameLayout {
    placed(
        text,
        &request_on(preferences, strings, SettingsTab::Terminals),
    )
}

fn command(layout: &FrameLayout) -> &ChoiceRow {
    let row = page(layout).row(SettingControl::StartupCommand);
    row.expect("the command row")
}

fn stepper(layout: &FrameLayout, control: SettingControl) -> &Stepper {
    page(layout).stepper(control).expect("the stepper")
}

fn close_switch(layout: &FrameLayout) -> &ToggleRow {
    let switch = page(layout).toggle(SettingControl::ConfirmClose);
    switch.expect("the close switch")
}

#[test]
fn without_a_command_the_row_offers_to_add_one() {
    let mut text = text_system();
    let french = Language::French.strings();
    let layout = terminals(&mut text, &Preferences::default(), french);
    let row = command(&layout);
    assert_eq!(row.chips.len(), 1);
    assert_eq!(row.chips[0].control, SettingControl::StartupCommand);
    let expected = text.layout(french.add, layout.metrics.label());
    assert_eq!(row.chips[0].label.width, expected.width);
    assert!(row.clear.is_none());
}

#[test]
fn a_saved_command_shows_itself_and_a_button_to_remove_it() {
    let mut text = text_system();
    let preferences = Preferences::default().with_startup_command("cd work");
    let strings = Language::English.strings();
    let layout = terminals(&mut text, &preferences, strings);
    let row = command(&layout);
    let expected = text.layout("cd work", layout.metrics.label());
    assert_eq!(row.chips[0].label.width, expected.width);
    let (clear, control) = row.clear.expect("a button to remove it");
    assert_eq!(control, SettingControl::ClearStartupCommand);
    assert!(row.chips[0].rect.right() <= clear.x);
}

#[test]
fn the_command_field_sits_in_its_row_and_keeps_the_clicks() {
    let mut text = text_system();
    let preferences = Preferences::default().with_startup_command("cd work");
    let mut layout = terminals(&mut text, &preferences, Language::English.strings());
    let control = SettingControl::StartupCommand;
    layout.place_setting_field(&mut text, control, &TextField::new("cd work"));
    let field = page(&layout).field_for(control).expect("the field");
    let row = command(&layout);
    assert!(row.line.rect.contains(field.rect.center()));
    assert!(field.rect.x >= row.line.label_x + row.line.label.width);
    let (clear, _) = row.clear.expect("a button to remove it");
    for point in [field.rect.center(), clear.center()] {
        assert_eq!(
            layout.hit(point, false),
            Some(Hit::Setting(SettingControl::StartupCommand))
        );
    }
}

#[test]
fn the_kept_lines_value_opens_a_field_over_its_stepper() {
    let mut text = text_system();
    let preferences = Preferences::default();
    let mut layout = terminals(&mut text, &preferences, Language::English.strings());
    let control = SettingControl::HistoryLines;
    let history = stepper(&layout, control).clone();
    assert_eq!(history.typed, Some(control));
    let value = history.value_rect.center();
    assert_eq!(layout.hit(value, false), Some(Hit::Setting(control)));
    layout.place_setting_field(&mut text, control, &TextField::new("2500"));
    let settings = page(&layout);
    let field = settings.field_for(control).expect("the field");
    assert!(settings.field_for(SettingControl::StartupCommand).is_none());
    assert!(history.line.rect.contains(field.rect.center()));
    for point in [history.smaller.center(), value, history.bigger.center()] {
        assert!(field.rect.contains(point));
        assert_eq!(layout.hit(point, false), Some(Hit::Setting(control)));
    }
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
    let layout = on_tab(&Preferences::default(), SettingsTab::Terminals);
    let switch = close_switch(&layout);
    assert!(switch.on && switch.enabled);
    let count = stepper(&layout, SettingControl::InstanceTabsFewer);
    assert!(count.line.rect.bottom() <= switch.line.rect.y);
    let center = switch.line.rect.center();
    let hit = Some(Hit::Setting(SettingControl::ConfirmClose));
    assert_eq!(layout.hit(center, false), hit);
    let off = Preferences::default().toggle_confirm_close();
    assert!(!close_switch(&on_tab(&off, SettingsTab::Terminals)).on);
}
