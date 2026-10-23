use trame::preferences::Preferences;
use trame::ui::frame::{SettingControl, SettingsTab};

use super::connections::{hosts, opened};
use super::{english, page, panel};

#[test]
fn the_tabs_sit_under_the_title_and_mark_the_open_one() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    let [general, connections] = &settings.tabs;
    assert_eq!(general.control, SettingControl::Tab(SettingsTab::General));
    assert_eq!(
        connections.control,
        SettingControl::Tab(SettingsTab::Connections)
    );
    assert!(general.selected && !connections.selected);
    assert!(general.rect.right() <= connections.rect.x);
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
