use trame::geometry::Rect;
use trame::input::TextField;
use trame::preferences::ConnectForm;
use trame::text::TextSystem;
use trame::ui::frame::{ColorsView, EditKind, HostsView, PickerView, SettingsTab};

use super::{editor_popup, message_popup, settings_popup, sized};
use crate::{TEAL, WIDTH, text_system};

fn all_but_settings(text: &mut TextSystem, height: f32) -> Vec<Rect> {
    let field = TextField::new("#408080");
    let form = ConnectForm::new();
    let colors = ColorsView {
        selected: None,
        custom: &[],
        first: 0,
        filled: false,
    };
    let picker = PickerView {
        hsv: TEAL,
        field: &field,
        editing: false,
    };
    let hosts = HostsView {
        hosts: &[],
        first: 0,
        form: Some(&form),
        changing: false,
    };
    vec![
        editor_popup(text, (WIDTH, height), EditKind::Color(colors)),
        editor_popup(text, (WIDTH, height), EditKind::Picker(picker)),
        editor_popup(text, (WIDTH, height), EditKind::Connect(hosts)),
        message_popup(text, (WIDTH, height)),
    ]
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
