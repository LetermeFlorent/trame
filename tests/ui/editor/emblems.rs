use trame::input::TextField;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::Emblem;
use trame::ui::frame::{EditAnchor, EditKind, EditTarget};
use trame::ui::{FrameLayout, Hit, UiMetrics};
use trame::workspace::Workspace;

use super::place;
use crate::{text_system, workspace};

fn second_instance(
    text: &mut TextSystem,
    workspace: &Workspace,
    anchor: EditAnchor,
    kind: EditKind,
) -> FrameLayout {
    let target = EditTarget::Instance(workspace.instances[1].id);
    place(text, workspace, target, anchor, kind)
}

fn collapsed() -> Workspace {
    let mut workspace = workspace();
    workspace.sidebar_open = false;
    workspace
}

#[test]
fn the_icon_row_offers_every_emblem_and_nothing_else() {
    let mut text = text_system();
    let kind = EditKind::Emblem(Some(Emblem::Code));
    let frame = second_instance(&mut text, &workspace(), EditAnchor::Sidebar, kind);
    let editor = frame.editor.as_ref().expect("an editor");
    assert_eq!(editor.emblem, Some(Emblem::Code));
    assert!(editor.field.is_none() && editor.slots.is_empty());
    let order: Vec<Emblem> = editor.emblems.iter().map(|slot| slot.emblem).collect();
    assert_eq!(order, Emblem::ALL.to_vec());
    for slot in &editor.emblems {
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
fn a_rail_icon_opens_the_rename_field_beside_the_rail() {
    let mut text = text_system();
    let name = TextField::new("name");
    let kind = EditKind::Name(&name);
    let frame = second_instance(&mut text, &collapsed(), EditAnchor::Rail, kind);
    let rail = frame.rail.as_ref().expect("the rail");
    let row = rail.slots[1].rect.center().y;
    let field = frame
        .editor
        .as_ref()
        .and_then(|editor| editor.field.as_ref());
    let field = field.expect("a field beside the rail").rect;
    assert!(field.x >= rail.rect.right());
    assert!(field.y <= row && field.bottom() >= row);
    assert!(field.width >= UiMetrics::SIDEBAR_MIN_WIDTH);
    assert_eq!(frame.hit(field.center(), false), Some(Hit::EditField));
}

#[test]
fn a_rail_icon_opens_the_same_icon_window_as_the_sidebar() {
    let mut text = text_system();
    let kind = EditKind::Emblem(Some(Emblem::Gear));
    let from_rail = second_instance(&mut text, &collapsed(), EditAnchor::Rail, kind);
    let editor = from_rail.editor.as_ref().expect("an editor from the rail");
    let popup = editor.popup.as_ref().expect("a popup");
    assert!(popup.rect.contains(editor.strip.center()));
    assert_eq!(editor.emblems.len(), Emblem::ALL.len());
    let from_sidebar = second_instance(&mut text, &workspace(), EditAnchor::Sidebar, kind);
    let other = from_sidebar
        .editor
        .as_ref()
        .and_then(|editor| editor.popup.as_ref());
    assert_eq!(other.map(|popup| popup.rect.width), Some(popup.rect.width));
}

#[test]
fn the_rail_anchor_needs_a_rail_and_an_instance() {
    let mut text = text_system();
    let kind = EditKind::Emblem(Some(Emblem::Grid));
    let open = second_instance(&mut text, &workspace(), EditAnchor::Rail, kind);
    assert!(open.editor.is_none());
    let tab = EditTarget::Tab(SessionId(1));
    let frame = place(&mut text, &collapsed(), tab, EditAnchor::Rail, kind);
    assert!(frame.editor.is_none());
}
