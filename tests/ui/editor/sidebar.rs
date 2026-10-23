use trame::input::TextField;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::ui::frame::{EditAnchor, EditKind, EditTarget, EditorRequest, SidebarItem};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use super::place;
use crate::{HEIGHT, text_system, workspace};

fn rename(text: &mut TextSystem, workspace: &Workspace, target: EditTarget) -> FrameLayout {
    let name = TextField::new("name");
    place(
        text,
        workspace,
        target,
        EditAnchor::Sidebar,
        EditKind::Name(&name),
    )
}

#[test]
fn the_sidebar_field_covers_an_instance_row() {
    let mut text = text_system();
    let workspace = workspace();
    let id = workspace.instances[0].id;
    let frame = rename(&mut text, &workspace, EditTarget::Instance(id));
    let field = frame
        .editor
        .as_ref()
        .and_then(|editor| editor.field.as_ref());
    let field = field.expect("a field").rect;
    let sidebar = frame.sidebar.as_ref().expect("the sidebar");
    let row = sidebar
        .rows
        .iter()
        .find(|row| row.item == SidebarItem::Instance(id))
        .expect("the row");
    assert!(field.y >= row.rect.y && field.bottom() <= row.rect.bottom());
    assert_eq!(frame.hit(field.center(), false), Some(Hit::EditField));
}

#[test]
fn the_sidebar_editor_can_target_a_tab_row() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = rename(&mut text, &workspace, EditTarget::Tab(SessionId(2)));
    assert!(frame.editor.is_some());
}

#[test]
fn an_editor_for_a_missing_target_is_dropped() {
    let mut text = text_system();
    let workspace = workspace();
    let hidden = rename(&mut text, &workspace, EditTarget::Tab(SessionId(3)));
    assert!(hidden.editor.is_none());
    let strip = EditAnchor::Strip(workspace.instances[0].active_pane);
    let missing = EditTarget::Tab(SessionId(9));
    let frame = place(&mut text, &workspace, missing, strip, EditKind::Color(None));
    assert!(frame.editor.is_none());
}

#[test]
fn near_the_bottom_edge_the_colours_open_above_the_row() {
    let mut text = text_system();
    let workspace = workspace();
    let mut frame = crate::frame(&mut text, &workspace);
    let sidebar = frame.sidebar.as_mut().expect("the sidebar");
    let row = &mut sidebar.rows[0];
    row.rect.y = HEIGHT - row.rect.height;
    let row_top = row.rect.y;
    let request = EditorRequest {
        target: EditTarget::Instance(workspace.instances[0].id),
        anchor: EditAnchor::Sidebar,
        kind: EditKind::Color(None),
    };
    frame.place_editor(&mut text, &request);
    let editor = frame.editor.as_ref().expect("an editor");
    assert!(editor.strip.bottom() <= row_top);
    assert!(editor.strip.y >= 0.0);
}
