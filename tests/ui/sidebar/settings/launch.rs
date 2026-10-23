use trame::input::TextField;
use trame::locale::Language;
use trame::preferences::{MAX_INSTANCE_TABS, Preferences, Recall};
use trame::ui::Hit;
use trame::ui::frame::SettingControl;

use super::{english, panel, with_settings};
use crate::text_system;

#[test]
fn the_three_switches_follow_the_preferences_in_order() {
    let layout = english(&Preferences::default());
    let toggles = &panel(&layout).toggles;
    let controls = toggles.each_ref().map(|toggle| toggle.control);
    assert_eq!(controls, Recall::ALL.map(SettingControl::Recall));
    for toggle in toggles {
        assert!(toggle.on && toggle.enabled);
        assert!(toggle.line.rect.contains(toggle.switch.center()));
    }
    for pair in toggles.windows(2) {
        assert!(pair[0].line.rect.bottom() <= pair[1].line.rect.y);
    }
}

#[test]
fn a_switch_that_depends_on_one_turned_off_is_greyed_and_ignores_clicks() {
    let preferences = Preferences::default().toggle(Recall::Instances);
    let layout = english(&preferences);
    let [instances, tabs, text] = &panel(&layout).toggles;
    assert!(!instances.on && instances.enabled);
    for toggle in [tabs, text] {
        assert!(!toggle.on && !toggle.enabled);
        assert_eq!(
            layout.hit(toggle.line.rect.center(), false),
            Some(Hit::SettingsPanel)
        );
    }
    let preferences = Preferences::default().toggle(Recall::Tabs);
    let layout = english(&preferences);
    let [instances, tabs, text] = &panel(&layout).toggles;
    assert!(instances.on && instances.enabled);
    assert!(!tabs.on && tabs.enabled);
    assert!(!text.on && !text.enabled);
}

#[test]
fn without_a_command_the_row_offers_to_add_one() {
    let mut text = text_system();
    let french = Language::French.strings();
    let layout = with_settings(&mut text, &Preferences::default(), french);
    let row = &panel(&layout).command;
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
    let layout = with_settings(&mut text, &preferences, strings);
    let row = &panel(&layout).command;
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
    let mut layout = with_settings(&mut text, &preferences, Language::English.strings());
    layout.place_command_field(&mut text, &TextField::new("cd work"));
    let settings = panel(&layout);
    let field = settings.command_field.as_ref().expect("the field");
    let line = settings.command.line.rect;
    assert!(line.contains(field.rect.center()));
    assert!(field.rect.x >= settings.command.line.label_x + settings.command.line.label.width);
    let (clear, _) = settings.command.clear.expect("a button to remove it");
    for point in [field.rect.center(), clear.center()] {
        assert_eq!(
            layout.hit(point, false),
            Some(Hit::Setting(SettingControl::StartupCommand))
        );
    }
}

#[test]
fn the_terminal_count_stepper_stops_at_its_limits() {
    let layout = english(&Preferences::default());
    let stepper = &panel(&layout).instance_tabs;
    assert!(!stepper.can_shrink && stepper.can_grow);
    let most = Preferences::default().step_instance_tabs(i32::from(MAX_INSTANCE_TABS));
    let layout = english(&most);
    let stepper = &panel(&layout).instance_tabs;
    assert!(stepper.can_shrink && !stepper.can_grow);
}
