use trame::geometry::Point;
use trame::locale::Language;
use trame::preferences::{ConnectForm, FormField};
use trame::text::TextSystem;
use trame::ui::frame::{EditAnchor, EditKind, EditTarget, FormFrame, SECRET_MASK};
use trame::ui::{FrameLayout, Hit, UiMetrics};
use trame::workspace::Workspace;

use super::place;
use crate::{text_system, workspace};

fn typed(entries: &[(FormField, &str)]) -> ConnectForm {
    let mut form = ConnectForm::new();
    for (field, text) in entries {
        form.focus_on(*field);
        form.focused_mut().insert(text);
    }
    form
}

fn on_sidebar(text: &mut TextSystem, workspace: &Workspace, form: &ConnectForm) -> FrameLayout {
    let target = EditTarget::Instance(workspace.instances[0].id);
    let strings = Language::French.strings();
    place(
        text,
        workspace,
        target,
        EditAnchor::Sidebar,
        EditKind::Form(form, strings),
    )
}

fn form_of(frame: &FrameLayout) -> &FormFrame {
    let editor = frame.editor.as_ref().expect("an editor");
    editor.form.as_ref().expect("a form")
}

#[test]
fn the_form_shows_four_labelled_fields_and_a_button_inside_the_window() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_sidebar(&mut text, &workspace, &ConnectForm::new());
    let form = form_of(&frame);
    assert!(frame.bounds.contains(form.rect.center()));
    assert!(
        form.rect.right() <= frame.bounds.right() && form.rect.bottom() <= frame.bounds.bottom()
    );
    let fields: Vec<FormField> = form.rows.iter().map(|row| row.field).collect();
    assert_eq!(fields, FormField::ALL);
    for pair in form.rows.windows(2) {
        assert!(pair[0].rect.bottom() <= pair[1].rect.y);
    }
    for row in &form.rows {
        assert!(row.label_x + row.label.width <= row.input.rect.x);
        assert!(row.input.rect.right() <= form.rect.right());
    }
    let last = form.rows.last().expect("a row");
    assert!(form.connect.y >= last.rect.bottom());
    assert!(form.connect.bottom() <= form.rect.bottom());
}

#[test]
fn only_the_focused_field_is_marked() {
    let mut text = text_system();
    let workspace = workspace();
    let form = typed(&[(FormField::Port, "22")]);
    let frame = on_sidebar(&mut text, &workspace, &form);
    let focused: Vec<FormField> = form_of(&frame)
        .rows
        .iter()
        .filter(|row| row.focused)
        .map(|row| row.field)
        .collect();
    assert_eq!(focused, [FormField::Port]);
}

#[test]
fn the_password_is_drawn_as_dots() {
    let mut text = text_system();
    let workspace = workspace();
    let form = typed(&[(FormField::Address, "box"), (FormField::Secret, "hunter2")]);
    let frame = on_sidebar(&mut text, &workspace, &form);
    let rows = &form_of(&frame).rows;
    let shown = |field: FormField| {
        let row = rows.iter().find(|row| row.field == field).expect("the row");
        row.input.label.clone()
    };
    let address = shown(FormField::Address);
    let secret = shown(FormField::Secret);
    let dots = text.layout(
        &SECRET_MASK.to_string().repeat(7),
        UiMetrics::new(1.0).label(),
    );
    assert_eq!(secret.width, dots.width);
    assert!(address.width > 0.0);
}

#[test]
fn each_part_of_the_form_answers_the_pointer() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_sidebar(&mut text, &workspace, &ConnectForm::new());
    let form = form_of(&frame);
    for row in &form.rows {
        let hit = frame.hit(row.input.rect.center(), false);
        assert_eq!(hit, Some(Hit::FormField(row.field)));
        let label = row.rect.x + 1.0;
        let on_label = Point::new(label, row.rect.center().y);
        assert_eq!(frame.hit(on_label, false), Some(Hit::EditStrip));
    }
    assert_eq!(
        frame.hit(form.connect.center(), false),
        Some(Hit::FormConnect)
    );
}
