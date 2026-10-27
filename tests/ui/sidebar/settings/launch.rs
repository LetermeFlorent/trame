use trame::preferences::{Preferences, Recall};
use trame::ui::frame::{SettingControl, ToggleRow};
use trame::ui::{FrameLayout, Hit};

use super::{english, page};

fn recalls(layout: &FrameLayout) -> [&ToggleRow; 3] {
    std::array::from_fn(|index| &page(layout).toggles[index])
}

#[test]
fn the_three_switches_follow_the_preferences_in_order() {
    let layout = english(&Preferences::default());
    let toggles = &page(&layout).toggles;
    let controls: Vec<_> = toggles.iter().map(|toggle| toggle.control).collect();
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
    let [instances, tabs, text] = recalls(&layout);
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
    let [instances, tabs, text] = recalls(&layout);
    assert!(instances.on && instances.enabled);
    assert!(!tabs.on && tabs.enabled);
    assert!(!text.on && !text.enabled);
}
