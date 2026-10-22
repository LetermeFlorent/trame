use trame::input::TextField;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::Swatch;
use trame::ui::frame::{EditAnchor, EditTarget, SidebarItem};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use super::request;
use crate::{HEIGHT, frame, text_system, workspace};

fn sidebar_editor(text: &mut TextSystem, workspace: &Workspace, target: EditTarget) -> FrameLayout {
    let mut frame = frame(text, workspace);
    let field = TextField::new("name");
    let anchor = EditAnchor::Sidebar;
    frame.place_editor(text, &request(target, anchor, &field, None));
    frame
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
