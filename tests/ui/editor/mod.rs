mod sidebar;
mod swatches;

use trame::geometry::Point;
use trame::input::TextField;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::Swatch;
use trame::ui::frame::{EditAnchor, EditTarget, EditorRequest};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use crate::{frame, text_system, workspace};

fn request<'a>(
    target: EditTarget,
    anchor: EditAnchor,
    field: &'a TextField,
    color: Option<Swatch>,
) -> EditorRequest<'a> {
    EditorRequest {
        target,
        anchor,
        field,
        color,
    }
}

fn strip_editor(text: &mut TextSystem, workspace: &Workspace, field: &TextField) -> FrameLayout {
    let mut frame = frame(text, workspace);
    let pane = frame.panes[0].id;
    let target = EditTarget::Tab(SessionId(2));
    frame.place_editor(text, &request(target, EditAnchor::Strip(pane), field, None));
    frame
}

#[test]
fn no_editor_is_placed_until_one_is_requested() {
    let mut text = text_system();
    assert!(frame(&mut text, &workspace()).editor.is_none());
}

#[test]
fn the_field_covers_the_edited_tab_in_the_strip() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = strip_editor(&mut text, &workspace, &TextField::new("tab 2"));
    let editor = frame.editor.as_ref().expect("an editor");
    let tab = frame.panes[0].strip.tabs[1].rect;
    assert!((editor.field.x - tab.x).abs() <= 1.0);
    assert!(editor.field.y >= tab.y && editor.field.bottom() <= tab.bottom());
    assert!((editor.field.width - tab.width).abs() <= 1.0);
    assert!(editor.strip.width >= editor.field.width);
    assert_eq!(
        frame.hit(editor.field.center(), false),
        Some(Hit::EditField)
    );
}

#[test]
fn a_point_outside_the_editor_hits_what_was_there_before() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = strip_editor(&mut text, &workspace, &TextField::new("tab 2"));
    let editor = frame.editor.as_ref().expect("an editor");
    let outside = Point::new(editor.strip.right() + 30.0, editor.strip.center().y);
    assert!(!editor.contains(outside));
    let plain = crate::frame(&mut text, &workspace);
    assert_eq!(frame.hit(outside, false), plain.hit(outside, false));
}

#[test]
fn the_caret_follows_the_text_and_stays_inside_the_field() {
    let mut text = text_system();
    let workspace = workspace();
    let mut field = TextField::new("abc");
    let at_end = strip_editor(&mut text, &workspace, &field)
        .editor
        .expect("an editor")
        .caret;
    field.home();
    let at_start = strip_editor(&mut text, &workspace, &field)
        .editor
        .expect("an editor")
        .caret;
    assert!(at_end.x > at_start.x);
    let long = TextField::new(&"w".repeat(100));
    let frame = strip_editor(&mut text, &workspace, &long);
    let editor = frame.editor.as_ref().expect("an editor");
    assert!(editor.caret.right() <= editor.text_area.right() + 1.0);
    assert!(editor.text_x < editor.text_area.x);
}
