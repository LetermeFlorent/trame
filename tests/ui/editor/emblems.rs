use trame::input::TextField;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::Emblem;
use trame::ui::frame::{EditAnchor, EditTarget, EditorRequest};
use trame::ui::{FrameLayout, Hit, UiMetrics};
use trame::workspace::Workspace;

use super::request;
use crate::{frame, text_system, workspace};

fn instance_editor(
    text: &mut TextSystem,
    workspace: &Workspace,
    anchor: EditAnchor,
    emblem: Emblem,
) -> FrameLayout {
    let mut frame = frame(text, workspace);
    let field = TextField::new("name");
    let target = EditTarget::Instance(workspace.instances[1].id);
    let request = EditorRequest {
        emblem: Some(emblem),
        ..request(target, anchor, &field, None)
    };
    frame.place_editor(text, &request);
    frame
}

fn collapsed() -> Workspace {
    let mut workspace = workspace();
    workspace.sidebar_open = false;
    workspace
}

#[test]
fn an_instance_editor_offers_every_emblem_under_the_colours() {
    let mut text = text_system();
    let frame = instance_editor(&mut text, &workspace(), EditAnchor::Sidebar, Emblem::Code);
    let editor = frame.editor.as_ref().expect("an editor");
    assert_eq!(editor.emblem, Some(Emblem::Code));
    let order: Vec<Emblem> = editor.emblems.iter().map(|slot| slot.emblem).collect();
    assert_eq!(order, Emblem::ALL.to_vec());
    let colours = editor.slots[0].rect;
    for slot in &editor.emblems {
        assert!(slot.rect.y > colours.bottom());
        assert!(editor.strip.contains(slot.rect.center()));
        assert!(slot.rect.right() <= editor.strip.right());
        assert!(slot.rect.contains(slot.icon.center()));
        assert_eq!(
            frame.hit(slot.rect.center(), false),
            Some(Hit::EditEmblem(slot.emblem))
        );
    }
}

#[test]
fn a_tab_editor_has_no_emblem_row() {
    let mut text = text_system();
    let workspace = workspace();
    let mut tab = frame(&mut text, &workspace);
    let field = TextField::new("tab");
    let target = EditTarget::Tab(SessionId(1));
    tab.place_editor(
        &mut text,
        &request(target, EditAnchor::Sidebar, &field, None),
    );
    let tab_editor = tab.editor.as_ref().expect("a tab editor");
    assert!(tab_editor.emblems.is_empty());
    assert_eq!(tab_editor.emblem, None);
    let instance = instance_editor(&mut text, &workspace, EditAnchor::Sidebar, Emblem::Grid);
    let instance_editor = instance.editor.as_ref().expect("an instance editor");
    assert!(instance_editor.strip.height > tab_editor.strip.height);
}

#[test]
fn a_rail_icon_opens_the_editor_beside_the_rail() {
    let mut text = text_system();
    let workspace = collapsed();
    let frame = instance_editor(&mut text, &workspace, EditAnchor::Rail, Emblem::Gear);
    let editor = frame.editor.as_ref().expect("an editor beside the rail");
    let rail = frame.rail.as_ref().expect("the rail");
    let slot = &rail.slots[1];
    assert!(editor.field.x >= rail.rect.right());
    assert!(
        editor.field.y <= slot.rect.center().y && editor.field.bottom() >= slot.rect.center().y
    );
    assert!(editor.field.width >= UiMetrics::SIDEBAR_MIN_WIDTH);
    assert_eq!(editor.emblems.len(), Emblem::ALL.len());
    assert_eq!(
        frame.hit(editor.field.center(), false),
        Some(Hit::EditField)
    );
}

#[test]
fn the_rail_anchor_needs_a_rail_and_an_instance() {
    let mut text = text_system();
    let open = instance_editor(&mut text, &workspace(), EditAnchor::Rail, Emblem::Grid);
    assert!(open.editor.is_none());
    let mut frame = frame(&mut text, &collapsed());
    let field = TextField::new("tab");
    let target = EditTarget::Tab(SessionId(1));
    frame.place_editor(&mut text, &request(target, EditAnchor::Rail, &field, None));
    assert!(frame.editor.is_none());
}
