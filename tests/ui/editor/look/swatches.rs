use trame::geometry::Point;
use trame::theme::Swatch;
use trame::ui::Hit;
use trame::ui::frame::SettingControl;

use crate::editor::{closing_popup, colors, on_strip};
use crate::{text_system, workspace};

#[test]
fn the_colours_open_in_a_window_like_the_settings() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, colors(None));
    let editor = frame.editor.as_ref().expect("an editor");
    let popup = editor.popup.as_ref().expect("a popup");
    assert_eq!(popup.backdrop.y, frame.title_bar.rect.bottom());
    assert_eq!(popup.backdrop.bottom(), frame.bounds.bottom());
    let (panel, backdrop) = (popup.rect.center(), popup.backdrop.center());
    assert!((panel.x - backdrop.x).abs() <= 1.0 && popup.rect.y >= popup.backdrop.y);
    assert!(popup.title.label.width > 0.0);
    assert!(popup.rect.contains(editor.strip.center()));
    assert!(editor.strip.y >= popup.title.rect.bottom());
    assert!(editor.field.is_none() && editor.emblems.is_empty());
    assert_eq!(editor.slots.len(), Swatch::ALL.len() + 1);
    for slot in &editor.slots {
        assert!(editor.strip.contains(slot.rect.center()));
    }
}

#[test]
fn the_colour_window_closes_from_its_cross_or_from_outside() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, colors(None));
    let popup = closing_popup(&frame);
    let title = Point::new(popup.title.label_x + 1.0, popup.title.rect.center().y);
    assert_eq!(frame.hit(title, false), Some(Hit::EditStrip));
}

#[test]
fn each_swatch_is_a_hit_and_the_empty_one_resets() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, colors(None));
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
    let frame = on_strip(&mut text, &workspace, colors(None));
    let editor = frame.editor.as_ref().expect("an editor");
    let (first, second) = (editor.slots[0].rect, editor.slots[1].rect);
    let between = Point::new((first.right() + second.x) / 2.0, first.center().y);
    assert_eq!(frame.hit(between, false), Some(Hit::EditStrip));
}

#[test]
fn the_selected_swatch_is_remembered_by_the_frame() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, colors(Some(Swatch::Blue)));
    assert_eq!(
        frame.editor.as_ref().map(|editor| editor.selected),
        Some(Some(Swatch::Blue))
    );
}

#[test]
fn the_coloured_background_switch_sits_between_the_swatches_and_my_colours() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, colors(Some(Swatch::Blue)));
    let editor = frame.editor.as_ref().expect("an editor");
    let popup = editor.popup.as_ref().expect("a popup");
    let fill = editor.fill.as_ref().expect("a switch");
    assert!(fill.enabled && !fill.on);
    assert!(fill.line.rect.y >= editor.strip.bottom());
    assert!(editor.sections[0].rect.y >= fill.line.rect.bottom());
    assert!(popup.rect.contains(fill.switch.center()));
    let pressed = Some(Hit::Setting(SettingControl::Fill));
    assert_eq!(frame.hit(fill.line.rect.center(), false), pressed);
    assert_eq!(frame.hit(fill.switch.center(), false), pressed);
}

#[test]
fn without_a_colour_the_switch_is_greyed_and_inert() {
    let mut text = text_system();
    let workspace = workspace();
    let frame = on_strip(&mut text, &workspace, colors(None));
    let editor = frame.editor.as_ref().expect("an editor");
    let fill = editor.fill.as_ref().expect("a switch");
    assert!(!fill.enabled);
    assert_eq!(
        frame.hit(fill.line.rect.center(), false),
        Some(Hit::EditStrip)
    );
}
