use trame::geometry::Rect;
use trame::text::TextSystem;
use trame::ui::frame::SettingsTab;

use super::{editor_popups, message_popup, settings_popup};
use crate::text_system;

const SMALL_WIDTH: f32 = 520.0;
const SMALL_HEIGHT: f32 = 320.0;
const SMALL: (f32, f32) = (SMALL_WIDTH, SMALL_HEIGHT);
const TABS: [SettingsTab; 7] = [
    SettingsTab::General,
    SettingsTab::Terminals,
    SettingsTab::Display,
    SettingsTab::System,
    SettingsTab::Instances,
    SettingsTab::Connections,
    SettingsTab::Shortcuts,
];

fn settings_popups(text: &mut TextSystem) -> Vec<(String, Rect)> {
    TABS.into_iter()
        .map(|tab| {
            (
                format!("{tab:?} settings"),
                settings_popup(text, SMALL, tab),
            )
        })
        .collect()
}

#[test]
fn a_popup_stays_inside_a_small_window() {
    let mut text = text_system();
    let mut popups = editor_popups(&mut text, SMALL);
    popups.extend(settings_popups(&mut text));
    popups.push(("message".to_string(), message_popup(&mut text, SMALL)));
    for (name, popup) in popups {
        let across = popup.x >= 0.0 && popup.right() <= SMALL_WIDTH;
        let down = popup.y >= 0.0 && popup.bottom() <= SMALL_HEIGHT;
        assert!(across && down, "the {name} popup reaches {popup:?}");
    }
}
