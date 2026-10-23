mod emblems;
mod form;
mod menu;
mod servers;
mod sidebar;
mod swatches;

use trame::geometry::Point;
use trame::input::TextField;
use trame::locale::Language;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::ui::frame::{EditAnchor, EditKind, EditTarget, EditorRequest};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use crate::{frame, text_system, workspace};

fn place(
    text: &mut TextSystem,
    workspace: &Workspace,
    target: EditTarget,
    anchor: EditAnchor,
    kind: EditKind,
) -> FrameLayout {
    let mut frame = frame(text, workspace);
    frame.place_editor(
        text,
        &EditorRequest {
            target,
            anchor,
            kind,
            strings: Language::English.strings(),
        },
    );
    frame
}

fn on_strip(text: &mut TextSystem, workspace: &Workspace, kind: EditKind) -> FrameLayout {
    let strip = EditAnchor::Strip(workspace.instances[0].active_pane);
    place(text, workspace, EditTarget::Tab(SessionId(2)), strip, kind)
}

#[test]
fn no_editor_is_placed_until_one_is_requested() {
    let mut text = text_system();
    assert!(frame(&mut text, &workspace()).editor.is_none());
}

#[test]
fn renaming_shows_only_a_field_over_the_edited_tab() {
    let mut text = text_system();
    let workspace = workspace();
    let name = TextField::new("tab 2");
    let frame = on_strip(&mut text, &workspace, EditKind::Name(&name));
    let editor = frame.editor.as_ref().expect("an editor");
    let field = editor.field.as_ref().expect("a field");
    let tab = frame.panes[0].strip.tabs[1].rect;
    assert!((field.rect.x - tab.x).abs() <= 1.0);
    assert!(field.rect.y >= tab.y && field.rect.bottom() <= tab.bottom());
    assert!((field.rect.width - tab.width).abs() <= 1.0);
    assert!(editor.strip.is_empty());
    assert!(editor.slots.is_empty() && editor.emblems.is_empty());
    assert_eq!(frame.hit(field.rect.center(), false), Some(Hit::EditField));
}

#[test]
fn a_point_outside_the_field_hits_what_was_there_before() {
    let mut text = text_system();
    let workspace = workspace();
    let name = TextField::new("tab 2");
    let frame = on_strip(&mut text, &workspace, EditKind::Name(&name));
    let field = frame
        .editor
        .as_ref()
        .and_then(|editor| editor.field.as_ref());
    let rect = field.expect("a field").rect;
    let outside = Point::new(rect.right() + 30.0, rect.center().y);
    let plain = crate::frame(&mut text, &workspace);
    assert_eq!(frame.hit(outside, false), plain.hit(outside, false));
}

#[test]
fn the_caret_follows_the_text_and_stays_inside_the_field() {
    let mut text = text_system();
    let workspace = workspace();
    let mut caret_of = |name: &TextField| {
        let frame = on_strip(&mut text, &workspace, EditKind::Name(name));
        let field = frame.editor.and_then(|editor| editor.field);
        field.expect("a field")
    };
    let mut name = TextField::new("abc");
    let at_end = caret_of(&name).caret;
    name.home();
    let at_start = caret_of(&name).caret;
    assert!(at_end.x > at_start.x);
    let long = caret_of(&TextField::new(&"w".repeat(100)));
    assert!(long.caret.right() <= long.text_area.right() + 1.0);
    assert!(long.text_x < long.text_area.x);
}
