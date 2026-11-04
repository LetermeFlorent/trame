use trame::geometry::Point;
use trame::input::TextField;
use trame::session::SessionId;
use trame::ui::Hit;
use trame::ui::frame::{EditAnchor, EditKind, EditTarget, EditorFrame, FieldFrame, SidebarItem};

use super::{on_strip, place};
use crate::{text_system, workspace};

fn selected(name: &str) -> TextField {
    let mut field = TextField::new(name);
    field.select_all();
    field
}

fn field_of(editor: Option<&EditorFrame>) -> &FieldFrame {
    editor
        .and_then(|editor| editor.field.as_ref())
        .expect("a field")
}

#[test]
fn a_tab_name_is_edited_in_place_with_the_label_font() {
    let mut text = text_system();
    let workspace = workspace();
    let name = selected("tab 2");
    let frame = on_strip(&mut text, &workspace, EditKind::Name(&name));
    let field = field_of(frame.editor.as_ref());
    let tab = &frame.panes[0].strip.tabs[1];
    assert!(!field.boxed);
    assert_eq!(field.text_x, tab.label_x);
    assert_eq!(field.label.style, tab.label.style);
    assert_eq!(field.rect, tab.rect);
    assert!(field.text_area.right() <= tab.close.x);
}

#[test]
fn the_whole_name_is_highlighted_until_something_is_typed() {
    let mut text = text_system();
    let workspace = workspace();
    let mut name = selected("tab 2");
    let frame = on_strip(&mut text, &workspace, EditKind::Name(&name));
    let field = field_of(frame.editor.as_ref());
    let highlight = field.selection.expect("a highlight");
    assert!((highlight.x - field.text_x).abs() <= 1.0);
    assert!((highlight.width - field.label.width).abs() <= 1.0);
    name.insert("w");
    let frame = on_strip(&mut text, &workspace, EditKind::Name(&name));
    assert!(field_of(frame.editor.as_ref()).selection.is_none());
}

#[test]
fn only_the_edited_tab_hides_its_own_label() {
    let mut text = text_system();
    let workspace = workspace();
    let name = selected("tab 2");
    let frame = on_strip(&mut text, &workspace, EditKind::Name(&name));
    let editor = frame.editor.as_ref().expect("an editor");
    let pane = workspace.instances[0].active_pane;
    assert!(editor.renames_tab(pane, SessionId(2)));
    assert!(!editor.renames_tab(pane, SessionId(1)));
    assert!(!editor.renames_row(SidebarItem::Tab(SessionId(2))));
}

#[test]
fn an_instance_row_is_edited_in_place_in_its_bold_font() {
    let mut text = text_system();
    let workspace = workspace();
    let id = workspace.instances[0].id;
    let name = selected("name");
    let target = EditTarget::Instance(id);
    let kind = EditKind::Name(&name);
    let frame = place(&mut text, &workspace, target, EditAnchor::Sidebar, kind);
    let field = field_of(frame.editor.as_ref());
    let sidebar = frame.sidebar.as_ref().expect("the sidebar");
    let item = SidebarItem::Instance(id);
    let row = sidebar.rows.iter().find(|row| row.item == item);
    let row = row.expect("the row");
    assert!(!field.boxed);
    assert_eq!(field.text_x, row.label_x);
    assert_eq!(field.label.style, row.label.style);
    assert!(field.text_area.right() <= row.close.x);
    assert!(
        frame
            .editor
            .as_ref()
            .is_some_and(|editor| editor.renames_row(item))
    );
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
