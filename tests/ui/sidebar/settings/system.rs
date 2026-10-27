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

#[cfg(windows)]
#[test]
fn one_update_switch_closes_the_page_and_is_on_by_default() {
    let toggles = switches(&Preferences::default());
    let auto = position(&toggles, SettingControl::AutoUpdate);
    assert_eq!(auto, toggles.len() - 1);
    assert!(toggles[auto].on && toggles[auto].enabled);
    let notify = position(&toggles, SettingControl::NotifyWaiting);
    assert_eq!(toggles[auto].line.label_x, toggles[notify].line.label_x);
    let toggles = switches(&Preferences {
        updates: trame::preferences::Updates { auto: false },
        ..Preferences::default()
    });
    let auto = position(&toggles, SettingControl::AutoUpdate);
    assert!(!toggles[auto].on && toggles[auto].enabled);
}

#[cfg(not(windows))]
#[test]
fn only_the_efficiency_switches_show_outside_windows() {
    let toggles = switches(&Preferences::default());
    let controls: Vec<_> = toggles.iter().map(|toggle| toggle.control).collect();
    let expected = [SettingControl::Efficiency, SettingControl::RestHiddenTabs];
    assert_eq!(controls, expected);
}
