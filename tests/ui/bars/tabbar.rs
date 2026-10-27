use trame::geometry::Rect;
use trame::render::Scene;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::{Appearance, Rgba, Swatch, Theme};
use trame::ui::chrome::draw_chrome;
use trame::ui::{Density, FrameLayout, PanelSide, UiMetrics};
use trame::workspace::Workspace;

use crate::paint::paint_with;
use crate::{frame, frame_in, text_system, workspace};

const EPSILON: f32 = 0.01;

fn coloured(first: Option<Swatch>, second: Option<Swatch>) -> Workspace {
    let mut workspace = workspace();
    workspace.set_tab_color(SessionId(1), first);
    workspace.set_tab_color(SessionId(2), second);
    workspace
}

fn paint(text: &mut TextSystem, frame: &FrameLayout, theme: &Theme) -> Scene {
    paint_with(text, frame, theme, None, draw_chrome)
}

fn painted_at(scene: &Scene, rect: Rect, color: Rgba) -> bool {
    scene
        .quads()
        .iter()
        .any(|quad| quad.rect[..2] == [rect.x, rect.y] && quad.color == color.to_array())
}

fn line_over(scene: &Scene, tab: Rect, color: Rgba) -> Option<Rect> {
    scene
        .quads()
        .iter()
        .filter(|quad| quad.color == color.to_array())
        .map(|quad| Rect::new(quad.rect[0], quad.rect[1], quad.rect[2], quad.rect[3]))
        .find(|line| line.y == tab.y && line.x >= tab.x && line.right() <= tab.right() + EPSILON)
}

#[test]
fn a_coloured_tab_keeps_its_line_when_another_tab_is_open() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let frame = frame(&mut text, &coloured(Some(Swatch::Blue), None));
    let scene = paint(&mut text, &frame, &theme);
    let tabs = &frame.panes[0].strip.tabs;
    assert!(!tabs[0].active);
    assert!(line_over(&scene, tabs[0].rect, theme.swatch(Swatch::Blue)).is_some());
    assert!(!painted_at(
        &scene,
        tabs[0].rect,
        theme.swatch_tint(Swatch::Blue)
    ));
}

#[test]
fn the_open_coloured_tab_is_filled_with_its_tint() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let frame = frame(&mut text, &coloured(None, Some(Swatch::Green)));
    let scene = paint(&mut text, &frame, &theme);
    let tab = &frame.panes[0].strip.tabs[1];
    assert!(tab.active && frame.panes[0].focused);
    assert!(painted_at(
        &scene,
        tab.rect,
        theme.swatch_tint(Swatch::Green)
    ));
    assert!(line_over(&scene, tab.rect, theme.swatch(Swatch::Green)).is_some());
}

#[test]
fn a_plain_tab_shows_no_line_until_it_is_open() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let frame = frame(&mut text, &coloured(None, None));
    let scene = paint(&mut text, &frame, &theme);
    let tabs = &frame.panes[0].strip.tabs;
    assert!(line_over(&scene, tabs[0].rect, theme.accent).is_none());
    assert!(line_over(&scene, tabs[1].rect, theme.accent).is_some());
}

#[test]
fn a_rounded_tab_keeps_its_colour_line_off_the_corners() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let metrics = UiMetrics::new(1.0).with_display(PanelSide::Left, Density::Spacious);
    let workspace = coloured(Some(Swatch::Blue), Some(Swatch::Green));
    let frame = frame_in(&mut text, &workspace, metrics);
    let scene = paint(&mut text, &frame, &theme);
    let pane = &frame.panes[0];
    assert!(pane.radius > 0.0);
    for (tab, swatch) in pane.strip.tabs.iter().zip([Swatch::Blue, Swatch::Green]) {
        let line = line_over(&scene, tab.rect, theme.swatch(swatch)).expect("a colour line");
        assert!(line.x >= tab.rect.x + pane.radius);
        assert!(line.right() <= tab.rect.right() - pane.radius + EPSILON);
    }
}

#[test]
fn the_bar_buttons_keep_room_above_and_below() {
    let mut text = text_system();
    let workspace = workspace();
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let frame = frame_in(&mut text, &workspace, UiMetrics::new(scale));
        let strip = &frame.panes[0].strip;
        for button in [strip.new_tab, strip.split] {
            let above = button.y - strip.rect.y;
            let below = strip.rect.bottom() - button.bottom();
            assert!(above >= 1.0 && below >= 1.0, "{scale}: {above} {below}");
            assert!((above - below).abs() <= 1.0, "{scale}: {above} {below}");
        }
    }
}
