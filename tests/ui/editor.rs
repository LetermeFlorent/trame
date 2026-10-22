use trame::geometry::Point;
use trame::input::TextField;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::Swatch;
use trame::ui::frame::{EditAnchor, EditTarget, EditorRequest, SidebarItem};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use crate::{HEIGHT, frame, text_system, workspace};

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

fn sidebar_editor(text: &mut TextSystem, workspace: &Workspace, target: EditTarget) -> FrameLayout {
    let mut frame = frame(text, workspace);
    let field = TextField::new("name");
    let anchor = EditAnchor::Sidebar;
    frame.place_editor(text, &request(target, anchor, &field, None));
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
    assert!(editor.field.width >= tab.width);
    assert_eq!(
        frame.hit(editor.field.center(), false),
        Some(Hit::EditField)
    );
}

#[test]
fn the_swatch_row_sits_right_under_the_field() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = strip_editor(&mut text, &workspace, &TextField::new("tab 2"));
    let editor = frame.editor.as_ref().expect("an editor");
    assert_eq!(editor.strip.y, editor.field.bottom());
    assert_eq!(editor.strip.x, editor.field.x);
    assert_eq!(editor.slots.len(), 9);
    for slot in &editor.slots {
        assert!(editor.strip.contains(slot.rect.center()));
        assert!(!editor.field.contains(slot.rect.center()));
    }
}

#[test]
fn each_swatch_is_a_hit_and_the_empty_one_resets() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = strip_editor(&mut text, &workspace, &TextField::new("tab 2"));
    let editor = frame.editor.as_ref().expect("an editor");
    for (slot, expected) in editor.slots.iter().zip(Swatch::ALL) {
        let hit = frame.hit(slot.rect.center(), false);
        assert_eq!(hit, Some(Hit::EditSwatch(Some(expected))));
    }
    let empty = editor.slots.last().expect("slots");
    assert_eq!(empty.color, None);
    assert_eq!(
        frame.hit(empty.rect.center(), false),
        Some(Hit::EditSwatch(None))
    );
}

#[test]
fn the_gaps_between_swatches_still_belong_to_the_editor() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = strip_editor(&mut text, &workspace, &TextField::new("tab 2"));
    let editor = frame.editor.as_ref().expect("an editor");
    let (first, second) = (editor.slots[0].rect, editor.slots[1].rect);
    let between = Point::new((first.right() + second.x) / 2.0, first.center().y);
    assert_eq!(frame.hit(between, false), Some(Hit::EditField));
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
fn the_sidebar_editor_covers_an_instance_row() {
    let mut text = text_system();
    let workspace = workspace();
    let id = workspace.instances[0].id;
    let frame = sidebar_editor(&mut text, &workspace, EditTarget::Instance(id));
    let editor = frame.editor.as_ref().expect("an editor");
    let sidebar = frame.sidebar.as_ref().expect("the sidebar");
    let row = sidebar
        .rows
        .iter()
        .find(|row| row.item == SidebarItem::Instance(id))
        .expect("the row");
    assert!(editor.field.y >= row.rect.y && editor.field.bottom() <= row.rect.bottom());
    assert_eq!(
        frame.hit(editor.field.center(), false),
        Some(Hit::EditField)
    );
    let slot = editor.slots[2].rect.center();
    assert_eq!(
        frame.hit(slot, false),
        Some(Hit::EditSwatch(Some(Swatch::Yellow)))
    );
}

#[test]
fn the_sidebar_editor_can_target_a_tab_row() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = sidebar_editor(&mut text, &workspace, EditTarget::Tab(SessionId(2)));
    assert!(frame.editor.is_some());
}

#[test]
fn an_editor_for_a_missing_target_is_dropped() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = sidebar_editor(&mut text, &workspace, EditTarget::Tab(SessionId(3)));
    assert!(frame.editor.is_none());
    let mut frame = crate::frame(&mut text, &workspace);
    let field = TextField::default();
    let target = EditTarget::Tab(SessionId(9));
    let pane = frame.panes[0].id;
    frame.place_editor(
        &mut text,
        &request(target, EditAnchor::Strip(pane), &field, None),
    );
    assert!(frame.editor.is_none());
}

#[test]
fn the_selected_swatch_is_remembered_by_the_frame() {
    let mut text = text_system();
    let workspace = workspace();
    let mut frame = crate::frame(&mut text, &workspace);
    let pane = frame.panes[0].id;
    let field = TextField::default();
    let target = EditTarget::Tab(SessionId(1));
    let anchor = EditAnchor::Strip(pane);
    let request = request(target, anchor, &field, Some(Swatch::Blue));
    frame.place_editor(&mut text, &request);
    assert_eq!(
        frame.editor.as_ref().map(|editor| editor.selected),
        Some(Some(Swatch::Blue))
    );
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

#[test]
fn near_the_bottom_edge_the_swatches_open_above_the_field() {
    let mut text = text_system();
    let workspace = workspace();
    let mut frame = crate::frame(&mut text, &workspace);
    let sidebar = frame.sidebar.as_mut().expect("the sidebar");
    let row = &mut sidebar.rows[0];
    row.rect.y = HEIGHT - row.rect.height;
    let id = workspace.instances[0].id;
    let field = TextField::new("x");
    let target = EditTarget::Instance(id);
    frame.place_editor(
        &mut text,
        &request(target, EditAnchor::Sidebar, &field, None),
    );
    let editor = frame.editor.as_ref().expect("an editor");
    assert!(editor.strip.bottom() <= editor.field.y);
    assert!(editor.strip.y >= 0.0);
}
