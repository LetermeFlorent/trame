use trame::geometry::Point;
use trame::input::TextField;
use trame::text::TextSystem;
use trame::theme::Hsv;
use trame::ui::frame::{EditKind, FormAction, PickerFrame, PickerPart, PickerView};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use super::on_strip;
use crate::{text_system, workspace};

const TEAL: Hsv = Hsv {
    hue: 0.5,
    saturation: 0.5,
    value: 0.5,
};

fn picker(text: &mut TextSystem, workspace: &Workspace, editing: bool) -> FrameLayout {
    let field = TextField::new("#408080");
    let view = PickerView {
        hsv: TEAL,
        field: &field,
        editing,
    };
    on_strip(text, workspace, EditKind::Picker(view))
}

fn picker_frame(frame: &FrameLayout) -> &PickerFrame {
    let editor = frame.editor.as_ref().expect("an editor");
    editor.picker.as_ref().expect("a picker")
}

#[test]
fn the_picker_lays_out_a_square_a_hue_bar_and_a_code_row() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = picker(&mut text, &workspace, false);
    let picker = picker_frame(&frame);
    let popup = frame
        .editor
        .as_ref()
        .and_then(|editor| editor.popup.as_ref());
    let popup = popup.expect("a popup");
    assert!(picker.square.y >= popup.title.rect.bottom());
    assert!(popup.rect.contains(picker.square.center()));
    assert!(picker.hue.x > picker.square.right());
    assert_eq!(picker.hue.y, picker.square.y);
    assert_eq!(picker.hue.height, picker.square.height);
    assert!(picker.preview.y > picker.square.bottom());
    assert!(picker.field.rect.x > picker.preview.right());
    assert!(popup.rect.contains(picker.field.rect.center()));
    assert_eq!(picker.hsv, TEAL);
}

#[test]
fn every_part_of_the_picker_is_a_hit() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = picker(&mut text, &workspace, true);
    let picker = picker_frame(&frame);
    let hit = |point| frame.hit(point, false);
    let part = |part| Some(Hit::Picker(part));
    assert_eq!(hit(picker.square.center()), part(PickerPart::Square));
    assert_eq!(hit(picker.hue.center()), part(PickerPart::Hue));
    assert_eq!(hit(picker.field.rect.center()), part(PickerPart::Field));
    let form = frame
        .editor
        .as_ref()
        .and_then(|editor| editor.form.as_ref());
    let actions: Vec<FormAction> = form
        .expect("a form")
        .buttons
        .iter()
        .map(|button| button.action)
        .collect();
    assert_eq!(actions, [FormAction::Cancel, FormAction::Apply]);
    for button in &form.expect("a form").buttons {
        let pressed = Some(Hit::FormButton(button.action));
        assert_eq!(hit(button.rect.center()), pressed);
        assert!(button.rect.x > picker.field.rect.right());
    }
}

#[test]
fn the_square_sets_saturation_and_value_and_keeps_the_hue() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = picker(&mut text, &workspace, false);
    let picker = picker_frame(&frame);
    let square = picker.square;
    let corner = Point::new(square.right(), square.y);
    let bright = picker.picked(PickerPart::Square, corner, TEAL);
    assert_eq!(
        (bright.saturation, bright.value, bright.hue),
        (1.0, 1.0, 0.5)
    );
    let below = Point::new(square.x - 40.0, square.bottom() + 40.0);
    let dark = picker.picked(PickerPart::Square, below, TEAL);
    assert_eq!((dark.saturation, dark.value), (0.0, 0.0));
}

#[test]
fn the_hue_bar_sets_only_the_hue() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = picker(&mut text, &workspace, false);
    let picker = picker_frame(&frame);
    let bar = picker.hue;
    let quarter = Point::new(bar.center().x, bar.y + bar.height / 4.0);
    let picked = picker.picked(PickerPart::Hue, quarter, TEAL);
    assert!((picked.hue - 0.25).abs() < 1e-4);
    assert_eq!((picked.saturation, picked.value), (0.5, 0.5));
    let field = picker.picked(PickerPart::Field, quarter, TEAL);
    assert_eq!(field, TEAL);
}
