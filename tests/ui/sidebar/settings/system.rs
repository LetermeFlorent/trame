use trame::preferences::Preferences;
use trame::ui::frame::{SettingControl, SettingsTab, ToggleRow};

use super::{on_tab, page};

fn switches(preferences: &Preferences) -> Vec<ToggleRow> {
    let layout = on_tab(preferences, SettingsTab::System);
    page(&layout).toggles.clone()
}

fn position(toggles: &[ToggleRow], control: SettingControl) -> usize {
    let found = toggles.iter().position(|toggle| toggle.control == control);
    found.expect("the switch")
}

#[test]
fn the_hidden_tabs_switch_sits_indented_under_efficiency_and_follows_it() {
    for efficiency in [true, false] {
        let toggles = switches(&Preferences {
            efficiency,
            ..Preferences::default()
        });
        let main = position(&toggles, SettingControl::Efficiency);
        let rest = position(&toggles, SettingControl::RestHiddenTabs);
        assert_eq!(rest, main + 1);
        assert!(toggles[rest].line.label_x > toggles[main].line.label_x);
        assert!(toggles[rest].on);
        assert_eq!(toggles[rest].enabled, efficiency);
    }
}

#[test]
fn the_system_page_has_no_update_switch() {
    let layout = on_tab(&Preferences::default(), SettingsTab::System);
    let controls: Vec<_> = page(&layout).toggles.iter().map(|t| t.control).collect();
    let mut expected = vec![SettingControl::Efficiency, SettingControl::RestHiddenTabs];
    if cfg!(windows) {
        expected.push(SettingControl::NotifyWaiting);
    }
    assert_eq!(controls, expected);
    assert!(page(&layout).sections.is_empty());
}
