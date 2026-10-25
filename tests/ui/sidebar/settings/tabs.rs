use trame::preferences::Preferences;
use trame::ui::frame::{SettingControl, SettingsTab};

use super::connections::{hosts, opened};
use super::{english, page, panel};

#[test]
fn the_tabs_sit_under_the_title_and_mark_the_open_one() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    let [general, instances, connections] = &settings.tabs;
    assert_eq!(general.control, SettingControl::Tab(SettingsTab::General));
    assert_eq!(
        instances.control,
        SettingControl::Tab(SettingsTab::Instances)
    );
    assert_eq!(
        connections.control,
        SettingControl::Tab(SettingsTab::Connections)
    );
    assert!(general.selected && !instances.selected && !connections.selected);
    assert!(general.rect.right() <= instances.rect.x);
    assert!(instances.rect.right() <= connections.rect.x);
    assert!(general.rect.y >= settings.popup.title.rect.bottom());
    assert!(page(&layout).sections[0].rect.y >= general.rect.bottom());
}

#[test]
fn the_window_keeps_its_size_from_one_tab_to_the_other() {
    let general = english(&Preferences::default());
    let saved = hosts(60);
    for layout in [opened(&[], 0, None), opened(&saved, 0, None)] {
        assert_eq!(panel(&layout).popup.rect, panel(&general).popup.rect);
    }
}

#[test]
fn the_instances_tab_holds_the_three_number_switches() {
    let mut text = crate::text_system();
    let preferences = Preferences::default();
    let mut request =
        super::general_request(&preferences, trame::locale::Language::English.strings());
    request.tab = SettingsTab::Instances;
    let layout = super::placed(&mut text, &request);
    let settings = panel(&layout);
    let trame::ui::frame::SettingsPage::Instances(page) = &settings.page else {
        panic!("the instances page");
    };
    let general = english(&preferences);
    assert_eq!(settings.popup.rect, panel(&general).popup.rect);
    for toggle in &page.toggles {
        assert!(settings.popup.rect.contains(toggle.line.rect.center()));
        assert!(toggle.enabled);
        let hit = layout.hit(toggle.line.rect.center(), false);
        assert_eq!(hit, Some(trame::ui::Hit::Setting(toggle.control)));
    }
    assert!(page.toggles[0].on && page.toggles[1].on && !page.toggles[2].on);
}
