use trame::locale::Language;
use trame::preferences::Preferences;
use trame::ui::Hit;
use trame::ui::frame::{SettingControl, SettingsPage, SettingsTab};

use super::connections::{hosts, opened};
use super::{english, on_tab, page, panel, with_settings};
use crate::text_system;

#[test]
fn the_seven_tabs_fit_on_one_line_under_the_title_and_mark_the_open_one() {
    let mut text = text_system();
    for language in [Language::French, Language::English] {
        let layout = with_settings(&mut text, &Preferences::default(), language.strings());
        let settings = panel(&layout);
        let controls = settings.tabs.each_ref().map(|tab| tab.control);
        assert_eq!(controls, SettingsTab::ALL.map(SettingControl::Tab));
        let open: Vec<_> = settings.tabs.iter().map(|tab| tab.selected).collect();
        assert_eq!(open, [true, false, false, false, false, false, false]);
        for pair in settings.tabs.windows(2) {
            assert!(pair[0].rect.right() <= pair[1].rect.x);
            assert_eq!(pair[0].rect.y, pair[1].rect.y);
        }
        let [first, .., last] = &settings.tabs;
        assert!(last.rect.right() <= settings.popup.close.right());
        assert!(first.rect.y >= settings.popup.title.rect.bottom());
        assert!(page(&layout).sections[0].rect.y >= first.rect.bottom());
    }
}

#[test]
fn the_window_keeps_its_size_from_one_tab_to_the_other() {
    let general = panel(&english(&Preferences::default())).popup.rect;
    let wide = english(&Preferences::default()).metrics.px(360.0);
    assert!(general.width > wide);
    for tab in SettingsTab::ALL {
        let layout = on_tab(&Preferences::default(), tab);
        assert_eq!(panel(&layout).popup.rect, general);
    }
    let saved = hosts(60);
    for layout in [opened(&[], 0, None), opened(&saved, 0, None)] {
        assert_eq!(panel(&layout).popup.rect, general);
    }
}

#[test]
fn the_instances_tab_holds_the_three_number_switches() {
    let layout = on_tab(&Preferences::default(), SettingsTab::Instances);
    let settings = panel(&layout);
    let SettingsPage::Instances(page) = &settings.page else {
        panic!("the instances page");
    };
    for toggle in &page.toggles {
        assert!(settings.popup.rect.contains(toggle.line.rect.center()));
        assert!(toggle.enabled);
        let hit = layout.hit(toggle.line.rect.center(), false);
        assert_eq!(hit, Some(Hit::Setting(toggle.control)));
    }
    assert!(page.toggles[0].on && page.toggles[1].on && !page.toggles[2].on);
}
