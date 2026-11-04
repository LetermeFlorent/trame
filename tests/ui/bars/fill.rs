use trame::geometry::{Point, Rect};
use trame::render::Scene;
use trame::session::SessionId;
use trame::theme::{Appearance, Rgba, Swatch, Theme};
use trame::ui::chrome::draw_chrome;
use trame::ui::frame::SidebarItem;
use trame::workspace::Workspace;

use super::painted_at;
use crate::paint::paint_with;
use crate::{frame, text_system, workspace};

const SWATCH: Swatch = Swatch::Teal;
const THEMES: [Appearance; 2] = [Appearance::Dark, Appearance::Light];

fn inked_inside(scene: &Scene, rect: Rect, color: Rgba) -> usize {
    let center = |rect: [f32; 4]| Point::new(rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0);
    scene
        .quads()
        .iter()
        .filter(|quad| quad.color == color.to_array() && rect.contains(center(quad.rect)))
        .count()
}

fn tab_filled(fill: bool) -> Workspace {
    let mut workspace = workspace();
    workspace.set_tab_color(SessionId(1), Some(SWATCH));
    workspace.set_tab_fill(SessionId(1), fill);
    workspace
}

fn instance_filled(fill: bool) -> Workspace {
    let mut workspace = workspace();
    let id = workspace.instances[0].id;
    workspace.set_instance_color(id, Some(SWATCH));
    workspace.set_instance_fill(id, fill);
    workspace
}

#[test]
fn a_filled_tab_paints_its_solid_colour_and_a_readable_label() {
    for appearance in THEMES {
        let theme = Theme::new(appearance);
        let mut text = text_system();
        let solid = theme.swatch(SWATCH);
        let [on, off] = [true, false].map(|fill| {
            let frame = frame(&mut text, &tab_filled(fill));
            let scene = paint_with(&mut text, &frame, &theme, None, draw_chrome);
            let tab = &frame.panes[0].strip.tabs[0];
            assert_eq!(tab.id, SessionId(1));
            let filled = painted_at(&scene, tab.rect, solid);
            (filled, inked_inside(&scene, tab.rect, solid.ink()))
        });
        assert!(on.0 && !off.0, "{appearance:?}");
        assert!(on.1 > off.1, "{appearance:?}");
    }
}

#[test]
fn the_switch_does_nothing_without_a_colour() {
    let mut workspace = workspace();
    workspace.set_tab_fill(SessionId(1), true);
    assert!(workspace.tab_fill(SessionId(1)));
    assert_eq!(workspace.tint_of(SessionId(1)), None);
}

#[test]
fn a_filled_instance_row_paints_its_solid_colour_and_a_readable_label() {
    for appearance in THEMES {
        let theme = Theme::new(appearance);
        let mut text = text_system();
        let solid = theme.swatch(SWATCH);
        let [on, off] = [true, false].map(|fill| {
            let workspace = instance_filled(fill);
            let frame = frame(&mut text, &workspace);
            let scene = paint_with(&mut text, &frame, &theme, None, draw_chrome);
            let item = SidebarItem::Instance(workspace.instances[0].id);
            let sidebar = frame.sidebar.as_ref().expect("a sidebar");
            let row = sidebar.rows.iter().find(|row| row.item == item);
            let row = row.expect("the instance row");
            let filled = painted_at(&scene, row.rect, solid);
            (filled, inked_inside(&scene, row.rect, solid.ink()))
        });
        assert!(on.0 && !off.0, "{appearance:?}");
        assert!(on.1 > off.1, "{appearance:?}");
    }
}

#[test]
fn the_terminal_is_tinted_only_when_the_switch_is_on() {
    for appearance in THEMES {
        let theme = Theme::new(appearance);
        for fill in [true, false] {
            let mut text = text_system();
            let frame = frame(&mut text, &instance_filled(fill));
            let pane = &frame.panes[0];
            assert_eq!(pane.tint, fill.then_some(SWATCH));
            let scene = paint_with(&mut text, &frame, &theme, None, draw_chrome);
            let open = pane.strip.tabs.iter().find(|tab| tab.active);
            let open = open.expect("an open tab");
            let tinted = theme.terminal_under(Some(SWATCH));
            assert_eq!(painted_at(&scene, open.rect, tinted), fill);
            assert_eq!(painted_at(&scene, open.rect, theme.terminal), !fill);
        }
    }
}

#[test]
fn a_tab_with_its_own_fill_wins_over_its_instance() {
    let mut workspace = instance_filled(true);
    workspace.set_tab_color(SessionId(2), Some(Swatch::Pink));
    assert_eq!(workspace.tint_of(SessionId(2)), Some(SWATCH));
    workspace.set_tab_fill(SessionId(2), true);
    assert_eq!(workspace.tint_of(SessionId(2)), Some(Swatch::Pink));
    assert_eq!(workspace.tint_of(SessionId(1)), Some(SWATCH));
    assert_eq!(workspace.tint_of(SessionId(3)), None);
}
