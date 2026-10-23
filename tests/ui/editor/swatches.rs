use trame::geometry::Point;
use trame::theme::Swatch;
use trame::ui::Hit;
use trame::ui::frame::EditKind;

use super::on_strip;
use crate::{text_system, workspace};

#[test]
fn the_colour_row_opens_right_under_the_tab_without_a_field() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, EditKind::Color(None));
    let editor = frame.editor.as_ref().expect("an editor");
    let tab = frame.panes[0].strip.tabs[1].rect;
    assert_eq!(editor.strip.y, tab.bottom());
    assert_eq!(editor.strip.x, tab.x);
    assert!(editor.field.is_none());
    assert!(editor.emblems.is_empty());
    assert_eq!(editor.slots.len(), Swatch::ALL.len() + 1);
    for slot in &editor.slots {
        assert!(editor.strip.contains(slot.rect.center()));
    }
}

#[test]
fn each_swatch_is_a_hit_and_the_empty_one_resets() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, EditKind::Color(None));
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
fn the_gaps_between_swatches_still_belong_to_the_strip() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, EditKind::Color(None));
    let editor = frame.editor.as_ref().expect("an editor");
    let (first, second) = (editor.slots[0].rect, editor.slots[1].rect);
    let between = Point::new((first.right() + second.x) / 2.0, first.center().y);
    assert_eq!(frame.hit(between, false), Some(Hit::EditStrip));
}

#[test]
fn the_selected_swatch_is_remembered_by_the_frame() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, EditKind::Color(Some(Swatch::Blue)));
    assert_eq!(
        frame.editor.as_ref().map(|editor| editor.selected),
        Some(Some(Swatch::Blue))
    );
}
