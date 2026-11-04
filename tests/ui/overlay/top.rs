use trame::geometry::Rect;
use trame::input::TextField;
use trame::locale::Language;
use trame::preferences::{ConnectForm, Preferences};
use trame::text::TextSystem;
use trame::theme::Hsv;
use trame::ui::frame::{
    ColorsView, EditAnchor, EditKind, EditTarget, EditorRequest, FormAction, HostsView,
    MessageRequest, PickerView, SettingsRequest, SettingsTab,
};
use trame::ui::{FrameLayout, UiMetrics};

use crate::{input, text_system, workspace};

fn layout(text: &mut TextSystem, height: f32) -> FrameLayout {
    let workspace = workspace();
    let mut input = input(&workspace, UiMetrics::new(1.0));
    input.width = 1200.0;
    input.height = height;
    FrameLayout::compute(text, &input)
}

fn edited(text: &mut TextSystem, height: f32, kind: EditKind) -> Rect {
    let workspace = workspace();
    let mut frame = layout(text, height);
    let request = EditorRequest {
        target: EditTarget::Instance(workspace.instances[0].id),
        anchor: EditAnchor::Sidebar,
        kind,
        strings: Language::English.strings(),
    };
    frame.place_editor(text, &request);
    frame.editor.and_then(|e| e.popup).expect("a popup").rect
}

fn message(text: &mut TextSystem, height: f32) -> Rect {
    let strings = Language::English.strings();
    let mut frame = layout(text, height);
    let request = MessageRequest {
        title: strings.close_window,
        lines: &["one", "two", "three"],
        actions: &[FormAction::Cancel, FormAction::Restart],
        strings,
    };
    frame.place_message(text, &request);
    frame.message.expect("a message").popup.rect
}

fn settings(text: &mut TextSystem, height: f32) -> Rect {
    let mut frame = layout(text, height);
    let request = SettingsRequest {
        preferences: &Preferences::default(),
        strings: Language::English.strings(),
        folder: None,
        tab: SettingsTab::General,
        hosts: None,
        shortcuts: None,
    };
    frame.place_settings(text, &request);
    frame.settings.expect("the settings").popup.rect
}

fn all_but_settings(text: &mut TextSystem, height: f32) -> Vec<Rect> {
    let field = TextField::new("#408080");
    let form = ConnectForm::new();
    let colors = ColorsView {
        selected: None,
        custom: &[],
        first: 0,
        filled: false,
    };
    let hsv = Hsv {
        hue: 0.5,
        saturation: 0.5,
        value: 0.5,
    };
    let picker = PickerView {
        hsv,
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
        edited(text, height, EditKind::Color(colors)),
        edited(text, height, EditKind::Picker(picker)),
        edited(text, height, EditKind::Connect(hosts)),
        message(text, height),
    ]
}

#[test]
fn every_popup_starts_at_the_settings_top() {
    let mut text = text_system();
    for height in [900.0, 700.0] {
        let top = settings(&mut text, height).y;
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
