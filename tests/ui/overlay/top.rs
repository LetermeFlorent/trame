use trame::geometry::Rect;
use trame::text::TextSystem;
use trame::ui::frame::SettingsTab;

use super::{editor_popups, message_popup, settings_popup, sized};
use crate::{WIDTH, text_system};

fn all_but_settings(text: &mut TextSystem, height: f32) -> Vec<Rect> {
    let size = (WIDTH, height);
    let mut popups: Vec<Rect> = editor_popups(text, size)
        .into_iter()
        .map(|(_, popup)| popup)
        .collect();
    popups.push(message_popup(text, size));
    popups
}

#[test]
fn every_popup_starts_at_the_settings_top() {
    let mut text = text_system();
    for height in [900.0, 700.0] {
        let top = settings_popup(&mut text, (WIDTH, height), SettingsTab::General).y;
        for popup in all_but_settings(&mut text, height) {
            assert_eq!(popup.y, top, "window height {height}");
        }
    }
}

#[test]
fn a_tall_popup_is_pushed_up_to_fit() {
    let mut text = text_system();
    let height = 360.0;
    for popup in all_but_settings(&mut text, height) {
        assert!(popup.bottom() <= height, "{popup:?}");
        assert!(popup.y >= 0.0);
    }
}

#[test]
fn a_small_window_keeps_a_gap_around_every_popup() {
    let mut text = text_system();
    for height in [360.0, 260.0] {
        let below_title = sized(&mut text, (WIDTH, height)).title_bar.rect.bottom();
        let mut popups = all_but_settings(&mut text, height);
        popups.push(settings_popup(
            &mut text,
            (WIDTH, height),
            SettingsTab::General,
        ));
        for popup in popups {
            assert!(popup.y >= below_title + 16.0, "top {popup:?}");
            let fits = height > 300.0;
            assert!(!fits || popup.bottom() <= height - 16.0, "bottom {popup:?}");
        }
    }
}
