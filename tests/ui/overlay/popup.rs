use trame::geometry::Rect;
use trame::input::TextField;
use trame::locale::Language;
use trame::preferences::{ConnectForm, Preferences};
use trame::text::TextSystem;
use trame::theme::Hsv;
use trame::ui::frame::{
    ColorsView, EditAnchor, EditKind, EditTarget, EditorRequest, EmblemView, FormAction, HostsView,
    MessageRequest, PickerView, SettingsRequest, SettingsTab,
};
use trame::ui::{FrameLayout, UiMetrics};
use trame::workspace::Workspace;

use crate::{input, text_system, workspace};

const SMALL_WIDTH: f32 = 520.0;
const SMALL_HEIGHT: f32 = 320.0;
const TEAL: Hsv = Hsv {
    hue: 0.5,
    saturation: 0.5,
    value: 0.5,
};
const TABS: [SettingsTab; 7] = [
    SettingsTab::General,
    SettingsTab::Terminals,
    SettingsTab::Display,
    SettingsTab::System,
    SettingsTab::Instances,
    SettingsTab::Connections,
    SettingsTab::Shortcuts,
];

fn small(text: &mut TextSystem, workspace: &Workspace) -> FrameLayout {
    let mut input = input(workspace, UiMetrics::new(1.0));
    input.width = SMALL_WIDTH;
    input.height = SMALL_HEIGHT;
    FrameLayout::compute(text, &input)
}

fn editor_popup(text: &mut TextSystem, kind: EditKind) -> Rect {
    let workspace = workspace();
    let mut layout = small(text, &workspace);
    let request = EditorRequest {
        target: EditTarget::Instance(workspace.instances[0].id),
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

fn editor_popups(text: &mut TextSystem) -> Vec<(String, Rect)> {
    let field = TextField::new("#408080");
    let form = ConnectForm::new();
    let colors = ColorsView {
        selected: None,
        custom: &[],
        first: 0,
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
        .map(|(name, kind)| (name.to_string(), editor_popup(text, kind)))
        .collect()
}

fn settings_popups(text: &mut TextSystem) -> Vec<(String, Rect)> {
    let preferences = Preferences::default();
    let mut popups = Vec::new();
    for tab in TABS {
        let mut layout = small(text, &workspace());
        let request = SettingsRequest {
            preferences: &preferences,
            strings: Language::English.strings(),
            folder: None,
            tab,
            hosts: None,
            shortcuts: None,
        };
        layout.place_settings(text, &request);
        let popup = layout.settings.expect("the settings").popup.rect;
        popups.push((format!("{tab:?} settings"), popup));
    }
    popups
}

fn message_popup(text: &mut TextSystem) -> (String, Rect) {
    let strings = Language::English.strings();
    let mut layout = small(text, &workspace());
    let request = MessageRequest {
        title: strings.close_window,
        lines: &["one", "two", "three"],
        actions: &[FormAction::Cancel, FormAction::Restart],
        strings,
    };
    layout.place_message(text, &request);
    let popup = layout.message.expect("a message").popup.rect;
    ("message".to_string(), popup)
}

#[test]
fn a_popup_stays_inside_a_small_window() {
    let mut text = text_system();
    let mut popups = editor_popups(&mut text);
    popups.extend(settings_popups(&mut text));
    popups.push(message_popup(&mut text));
    for (name, popup) in popups {
        let across = popup.x >= 0.0 && popup.right() <= SMALL_WIDTH;
        let down = popup.y >= 0.0 && popup.bottom() <= SMALL_HEIGHT;
        assert!(across && down, "the {name} popup reaches {popup:?}");
    }
}
