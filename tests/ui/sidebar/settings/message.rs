use trame::locale::Language;
use trame::preferences::Preferences;
use trame::text::TextSystem;
use trame::ui::frame::{FormAction, MessageFrame, MessageRequest, fit_text_start};
use trame::ui::{FrameLayout, Hit};

use super::{general_request, panel};
use crate::{frame, text_system, workspace};

fn showing(text: &mut TextSystem, line: &str, actions: &[FormAction]) -> FrameLayout {
    let strings = Language::English.strings();
    let mut layout = frame(text, &workspace());
    let request = MessageRequest {
        title: strings.close_window,
        line,
        actions,
        strings,
    };
    layout.place_message(text, &request);
    layout
}

fn confirming(text: &mut TextSystem) -> FrameLayout {
    showing(
        text,
        "3 tabs will close.",
        &[FormAction::Cancel, FormAction::Close],
    )
}

fn dialog(layout: &FrameLayout) -> &MessageFrame {
    layout.message.as_ref().expect("the message box")
}

#[test]
fn the_confirmation_offers_cancel_then_close_and_each_answers_where_drawn() {
    let layout = confirming(&mut text_system());
    let confirm = dialog(&layout);
    let actions: Vec<_> = confirm
        .form
        .buttons
        .iter()
        .map(|button| button.action)
        .collect();
    assert_eq!(actions, [FormAction::Cancel, FormAction::Close]);
    for button in &confirm.form.buttons {
        let center = button.rect.center();
        assert!(confirm.popup.rect.contains(center));
        assert_eq!(
            layout.hit(center, false),
            Some(Hit::FormButton(button.action))
        );
    }
    let close = confirm.popup.close.center();
    assert_eq!(layout.hit(close, false), Some(Hit::EditClose));
    let line = confirm.line.rect;
    assert_eq!(layout.hit(line.center(), false), Some(Hit::Dialog));
    assert!(line.bottom() <= confirm.form.rect.y);
}

#[test]
fn outside_the_confirmation_only_the_title_bar_answers() {
    let layout = confirming(&mut text_system());
    let row = layout.sidebar.as_ref().expect("the sidebar").rows[0].rect;
    assert_eq!(layout.hit(row.center(), false), Some(Hit::Backdrop));
    let controls = layout.title_bar.controls.expect("window buttons");
    assert_eq!(layout.hit(controls.close.center(), false), Some(Hit::Close));
}

#[test]
fn the_confirmation_answers_before_an_open_settings_window() {
    let mut text = text_system();
    let mut layout = confirming(&mut text);
    let preferences = Preferences::default();
    let request = general_request(&preferences, Language::English.strings());
    layout.place_settings(&mut text, &request);
    let confirm = dialog(&layout);
    for button in &confirm.form.buttons {
        let hit = layout.hit(button.rect.center(), false);
        assert_eq!(hit, Some(Hit::FormButton(button.action)));
    }
    let settings_close = panel(&layout).popup.close.center();
    assert!(!confirm.popup.rect.contains(settings_close));
    assert_eq!(layout.hit(settings_close, false), Some(Hit::Backdrop));
}

#[test]
fn a_long_folder_keeps_its_end_and_change_stays_beside_the_filled_apply() {
    let mut text = text_system();
    let deep = format!("C:/{}final", "nested/".repeat(30));
    let layout = showing(&mut text, &deep, &[FormAction::Change, FormAction::Apply]);
    let message = dialog(&layout);
    let style = layout.metrics.label();
    let shown = message.line.label.width;
    assert!(shown < text.layout(&deep, style).width);
    assert!(shown <= message.line.rect.width);
    assert!(fit_text_start(&mut text, &deep, style, shown).ends_with("nested/final"));
    let filled: Vec<_> = message
        .form
        .buttons
        .iter()
        .map(|button| button.action.is_primary())
        .collect();
    assert_eq!(filled, [false, true]);
}
