use trame::geometry::Point;
use trame::text::TextSystem;
use trame::theme::Swatch;
use trame::ui::frame::{ColorsView, EditKind, FormAction, ListControl};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use super::on_strip;
use crate::{text_system, workspace};

fn custom(count: u32) -> Vec<Swatch> {
    (0..count)
        .map(|index| Swatch::custom(0x102030 + index * 0x111111))
        .collect()
}

fn with_custom(text: &mut TextSystem, workspace: &Workspace, custom: &[Swatch]) -> FrameLayout {
    let view = ColorsView {
        selected: custom.last().copied(),
        custom,
        first: 0,
    };
    on_strip(text, workspace, EditKind::Color(view))
}

#[test]
fn my_colours_are_listed_under_the_presets_with_their_dot() {
    let mut text = text_system();
    let workspace = workspace();
    let colors = custom(2);
    let frame = with_custom(&mut text, &workspace, &colors);
    let editor = frame.editor.as_ref().expect("an editor");
    let list = editor.list.as_ref().expect("a list");
    assert_eq!(list.rows.len(), 2);
    assert_eq!(editor.sections.len(), 1);
    assert!(editor.sections[0].rect.y >= editor.strip.bottom());
    assert!(list.rect.y >= editor.sections[0].rect.bottom());
    assert_eq!(editor.dots.len(), 2);
    for (dot, row) in editor.dots.iter().zip(&list.rows) {
        assert_eq!(dot.color, Some(colors[row.index]));
        assert!(row.line.rect.contains(dot.rect.center()));
        assert!(row.line.label_x > dot.rect.right());
    }
    assert_eq!(editor.selected, Some(colors[1]));
}

#[test]
fn each_listed_colour_can_be_picked_changed_or_forgotten() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = with_custom(&mut text, &workspace, &custom(2));
    let list = frame
        .editor
        .as_ref()
        .and_then(|editor| editor.list.as_ref());
    let row = &list.expect("a list").rows[1];
    let hit = |point| frame.hit(point, false);
    let label = Point::new(row.line.label_x + 1.0, row.line.rect.center().y);
    assert_eq!(hit(label), Some(Hit::List(ListControl::Pick(1))));
    let change = Some(Hit::List(ListControl::Change(1)));
    assert_eq!(hit(row.change.center()), change);
    let forget = row.forget.expect("a cross");
    assert_eq!(
        hit(forget.center()),
        Some(Hit::List(ListControl::Forget(1)))
    );
}

#[test]
fn the_add_button_sits_under_the_list() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = with_custom(&mut text, &workspace, &custom(1));
    let editor = frame.editor.as_ref().expect("an editor");
    let form = editor.form.as_ref().expect("a form");
    assert_eq!(form.buttons.len(), 1);
    let add = &form.buttons[0];
    assert_eq!(add.action, FormAction::Add);
    let list = editor.list.as_ref().expect("a list");
    assert!(add.rect.y >= list.rect.bottom());
    let popup = editor.popup.as_ref().expect("a popup");
    assert!(popup.rect.contains(add.rect.center()));
    let hit = frame.hit(add.rect.center(), false);
    assert_eq!(hit, Some(Hit::FormButton(FormAction::Add)));
}

#[test]
fn without_custom_colours_only_the_caption_and_add_remain() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = with_custom(&mut text, &workspace, &[]);
    let editor = frame.editor.as_ref().expect("an editor");
    assert!(editor.list.is_none() && editor.dots.is_empty());
    assert_eq!(editor.sections.len(), 1);
    assert!(editor.form.is_some());
}

#[test]
fn a_full_list_scrolls_and_hides_the_add_button() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = with_custom(&mut text, &workspace, &custom(8));
    let editor = frame.editor.as_ref().expect("an editor");
    assert!(editor.form.is_none());
    let list = editor.list.as_ref().expect("a list");
    assert!(list.rows.len() < 8);
    assert!(list.bar.is_some());
    assert_eq!(editor.dots.len(), list.rows.len());
}
