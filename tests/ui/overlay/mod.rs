mod loader;
mod popup;
mod tooltip;
mod top;

use trame::geometry::Rect;
use trame::input::TextField;
use trame::locale::Language;
use trame::preferences::{ConnectForm, Preferences};
use trame::text::TextSystem;
use trame::ui::frame::{
    ColorsView, EditAnchor, EditKind, EditTarget, EditorRequest, EmblemView, FormAction, HostsView,
    MessageRequest, PickerView, SettingsRequest, SettingsTab,
};
use trame::ui::{FrameLayout, UiMetrics};

use crate::{TEAL, input, workspace};

fn sized(text: &mut TextSystem, (width, height): (f32, f32)) -> FrameLayout {
    let workspace = workspace();
    let mut input = input(&workspace, UiMetrics::new(1.0));
    input.width = width;
    input.height = height;
    FrameLayout::compute(text, &input)
}

fn editor_popups(text: &mut TextSystem, size: (f32, f32)) -> Vec<(String, Rect)> {
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
    let icons = EmblemView {
        selected: None,
        letters: &field,
    };
    let hosts = HostsView {
        hosts: &[],
        first: 0,
        form: Some(&form),
        changing: false,
    };
    let kinds = [
        ("colour", EditKind::Color(colors)),
        ("picker", EditKind::Picker(picker)),
        ("icon", EditKind::Emblem(icons)),
        ("connection", EditKind::Connect(hosts)),
    ];
    kinds
        .into_iter()
        .map(|(name, kind)| (name.to_string(), editor_popup(text, size, kind)))
        .collect()
}

fn editor_popup(text: &mut TextSystem, size: (f32, f32), kind: EditKind) -> Rect {
    let mut layout = sized(text, size);
    let request = EditorRequest {
        target: EditTarget::Instance(workspace().instances[0].id),
        anchor: EditAnchor::Sidebar,
        kind,
        strings: Language::English.strings(),
    };
    layout.place_editor(text, &request);
    layout
        .editor
        .and_then(|editor| editor.popup)
        .expect("a popup")
        .rect
}

fn message_popup(text: &mut TextSystem, size: (f32, f32)) -> Rect {
    let mut layout = sized(text, size);
    let strings = Language::English.strings();
    let request = MessageRequest {
        title: strings.close_window,
        lines: &["one", "two", "three"],
        actions: &[FormAction::Cancel, FormAction::Restart],
        strings,
    };
    layout.place_message(text, &request);
    layout.message.expect("a message").popup.rect
}

fn settings_popup(text: &mut TextSystem, size: (f32, f32), tab: SettingsTab) -> Rect {
    let mut layout = sized(text, size);
    let request = SettingsRequest {
        preferences: &Preferences::default(),
        strings: Language::English.strings(),
        folder: None,
        tab,
        hosts: None,
        shortcuts: None,
    };
    layout.place_settings(text, &request);
    layout.settings.expect("the settings").popup.rect
}
