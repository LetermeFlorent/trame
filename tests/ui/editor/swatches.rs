use trame::geometry::Point;
use trame::input::TextField;
use trame::session::SessionId;
use trame::theme::Swatch;
use trame::ui::Hit;
use trame::ui::frame::{EditAnchor, EditTarget};

use super::{request, strip_editor};
use crate::{text_system, workspace};

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
