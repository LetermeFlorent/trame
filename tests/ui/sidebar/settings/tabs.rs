use trame::locale::{Language, Strings};
use trame::preferences::Preferences;
use trame::ui::frame::{SettingControl, SettingsPage, SettingsTab};
use trame::ui::{FrameLayout, Hit, UiMetrics};

use super::connections::{hosts, opened};
use super::{english, general_request, on_tab, page, panel, with_settings};
use crate::{input, text_system, workspace};

const SMALLEST_WIDTH: f32 = 520.0;

fn tab_names(strings: &Strings) -> [&str; 7] {
    [
        strings.general,
        strings.terminals,
        strings.display,
        strings.system_section,
        strings.instances_section,
        strings.connections_tab,
        strings.keys.tab,
    ]
}

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
fn the_window_keeps_its_width_and_top_from_one_tab_to_the_other() {
    let full = panel(&on_tab(&Preferences::default(), SettingsTab::Shortcuts))
        .popup
        .rect;
    let wide = english(&Preferences::default()).metrics.px(360.0);
    assert!(full.width > wide);
    for tab in SettingsTab::ALL {
        let layout = on_tab(&Preferences::default(), tab);
        let rect = panel(&layout).popup.rect;
        assert_eq!((rect.x, rect.y, rect.width), (full.x, full.y, full.width));
        assert!(rect.height <= full.height, "{tab:?}");
    }
    let saved = hosts(60);
    let (empty, long) = (opened(&[], 0, None), opened(&saved, 0, None));
    let (empty, long) = (panel(&empty).popup.rect, panel(&long).popup.rect);
    assert_eq!((long.x, long.y, long.width), (full.x, full.y, full.width));
    assert!(empty.height < long.height);
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

#[test]
fn the_seven_tabs_keep_their_names_inside_the_smallest_window() {
    let mut text = text_system();
    let workspace = workspace();
    let mut input = input(&workspace, UiMetrics::new(1.0));
    input.width = SMALLEST_WIDTH;
    let preferences = Preferences::default();
    for language in [Language::French, Language::English] {
        let strings = language.strings();
        let mut layout = FrameLayout::compute(&mut text, &input);
        layout.place_settings(&mut text, &general_request(&preferences, strings));
        let settings = panel(&layout);
        let last = settings.tabs.last().expect("the tabs");
        assert!(
            last.rect.right() <= settings.popup.close.right(),
            "{language:?}"
        );
        for (tab, name) in settings.tabs.iter().zip(tab_names(strings)) {
            let whole = text.layout(name, layout.metrics.label());
            assert_eq!(tab.label.width, whole.width, "{name}");
        }
    }
}
