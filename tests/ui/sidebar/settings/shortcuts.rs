use trame::input::{ACTIONS, Combo, Command, Keymap, Overrides};
use trame::locale::{ACTION_SLOT, Language};
use trame::preferences::Preferences;
use trame::text::TextSystem;
use trame::ui::frame::{
    KeyControl, KeyNotice, ListControl, SettingControl, SettingsRequest, SettingsTab,
    ShortcutsPage, ShortcutsView, key_label,
};
use trame::ui::{FrameLayout, Hit, Platform};

use super::{english, general_request, panel, placed};
use crate::text_system;

fn opened(text: &mut TextSystem, preferences: &Preferences, view: ShortcutsView) -> FrameLayout {
    let request = SettingsRequest {
        tab: SettingsTab::Shortcuts,
        shortcuts: Some(view),
        ..general_request(preferences, Language::English.strings())
    };
    placed(text, &request)
}

fn page(layout: &FrameLayout) -> &ShortcutsPage {
    panel(layout).shortcuts().expect("the shortcuts page")
}

fn copy() -> usize {
    ACTIONS
        .iter()
        .position(|command| *command == Command::Copy)
        .unwrap()
}

fn editing(action: usize) -> ShortcutsView {
    ShortcutsView {
        action: Some(action),
        ..ShortcutsView::default()
    }
}

fn shown_keys(layout: &FrameLayout, preferences: &Preferences, action: usize) -> Vec<String> {
    let platform = layout.metrics.platform;
    let strings = Language::English.strings().keys;
    let keymap = Keymap::new(platform, &preferences.shortcuts);
    let keys = keymap.keys(action);
    keys.iter()
        .map(|combo| key_label(combo, strings, platform))
        .collect()
}

#[test]
fn the_tab_lists_every_action_with_its_keys() {
    let mut text = text_system();
    let preferences = Preferences::default();
    let layout = opened(&mut text, &preferences, ShortcutsView::default());
    let page = page(&layout);
    let general = english(&preferences);
    let (list, form) = (panel(&layout).popup.rect, panel(&general).popup.rect);
    assert_eq!((list.x, list.y, list.width), (form.x, form.y, form.width));
    assert!(list.height > form.height);
    assert!(page.header.chips.is_empty());
    let list = page.list.as_ref().expect("the action list");
    assert!(list.bar.is_some());
    assert_eq!(list.rows.len(), page.details.len());
    let (row, detail) = (&list.rows[0], &page.details[0]);
    let keys = shown_keys(&layout, &preferences, 0).join(", ");
    let expected = text.layout(&keys, layout.metrics.label());
    assert_eq!(detail.line.label.width, expected.width);
    assert!(!detail.changed);
    assert!(row.line.label_x + row.line.label.width < detail.line.label_x);
    assert!(detail.line.label_x + detail.line.label.width <= row.change.x);
    let pick = layout.hit(row.line.rect.center(), false);
    assert_eq!(pick, Some(Hit::List(ListControl::Pick(0))));
    let change = layout.hit(row.change.center(), false);
    assert_eq!(change, Some(Hit::List(ListControl::Change(0))));
}

#[test]
fn changed_keys_stand_out_and_can_all_go_back() {
    let f9 = Combo::parse("F9").unwrap();
    let overrides = Overrides::default().with_keys(0, vec![f9], Platform::current());
    let preferences = Preferences {
        shortcuts: overrides,
        ..Preferences::default()
    };
    let layout = opened(&mut text_system(), &preferences, ShortcutsView::default());
    let page = page(&layout);
    assert!(page.details[0].changed && !page.details[1].changed);
    let chip = &page.header.chips[0];
    assert_eq!(chip.control, SettingControl::Keys(KeyControl::ResetAll));
    let hit = layout.hit(chip.rect.center(), false);
    assert_eq!(hit, Some(Hit::Setting(chip.control)));
}

#[test]
fn an_action_shows_its_keys_with_ways_to_remove_add_and_go_back() {
    let mut text = text_system();
    let preferences = Preferences::default();
    let layout = opened(&mut text, &preferences, editing(copy()));
    let page = page(&layout);
    assert!(page.list.is_none() && page.hint.is_none() && page.notice.is_none());
    let keys = shown_keys(&layout, &preferences, copy());
    assert_eq!(page.keys.len(), keys.len());
    for (line, key) in page.keys.iter().zip(&keys) {
        let expected = text.layout(key, layout.metrics.label());
        assert_eq!(line.line.label.width, expected.width);
        let remove = SettingControl::Keys(KeyControl::Remove(line.index));
        let hit = layout.hit(line.remove.center(), false);
        assert_eq!(hit, Some(Hit::Setting(remove)));
    }
    let footer = page.footer.iter().flat_map(|row| &row.chips);
    let controls: Vec<SettingControl> = page
        .header
        .chips
        .iter()
        .chain(footer)
        .map(|chip| chip.control)
        .collect();
    let expected = [KeyControl::Back, KeyControl::Add].map(SettingControl::Keys);
    assert_eq!(controls, expected);
}

#[test]
fn while_waiting_for_keys_it_asks_for_them_and_says_why_one_is_refused() {
    let mut text = text_system();
    let view = ShortcutsView {
        capturing: true,
        notice: Some(KeyNotice::Taken(0)),
        ..editing(copy())
    };
    let layout = opened(&mut text, &Preferences::default(), view);
    let page = page(&layout);
    let strings = Language::English.strings().keys;
    let style = layout.metrics.label();
    assert!(page.capturing && page.footer.is_none());
    let hint = page.hint.as_ref().expect("the hint");
    assert_eq!(
        hint.label.width,
        text.layout(strings.press_keys, style).width
    );
    let notice = page.notice.as_ref().expect("the notice");
    let taken = strings.taken_by.replace(ACTION_SLOT, strings.actions[0]);
    assert_eq!(notice.label.width, text.layout(&taken, style).width);
    assert!(panel(&layout).popup.rect.contains(notice.rect.center()));
}
