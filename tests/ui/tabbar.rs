use trame::geometry::Rect;
use trame::render::Scene;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::{Appearance, Rgba, Swatch, Theme};
use trame::ui::chrome::{ChromeInput, draw_chrome};
use trame::ui::{FrameLayout, FrameView, Motion};
use trame::workspace::Workspace;

use crate::{frame, text_system, workspace};

fn coloured(first: Option<Swatch>, second: Option<Swatch>) -> Workspace {
    let mut workspace = workspace();
    workspace.set_tab_color(SessionId(1), first);
    workspace.set_tab_color(SessionId(2), second);
    workspace
}

fn paint(text: &mut TextSystem, frame: &FrameLayout, theme: &Theme) -> Scene {
    let view = FrameView {
        hover: None,
        pressed: None,
        window_focused: true,
    };
    let motion = Motion::default();
    let input = ChromeInput {
        theme,
        frame,
        view: &view,
        motion: &motion,
    };
    let mut scene = Scene::new(frame.bounds.width, frame.bounds.height);
    draw_chrome(&mut scene, text, &input);
    scene
}

fn painted_at(scene: &Scene, rect: Rect, color: Rgba) -> bool {
    scene
        .quads()
        .iter()
        .any(|quad| quad.rect[..2] == [rect.x, rect.y] && quad.color == color.to_array())
}

#[test]
fn a_coloured_tab_keeps_its_line_when_another_tab_is_open() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let frame = frame(&mut text, &coloured(Some(Swatch::Blue), None));
    let scene = paint(&mut text, &frame, &theme);
    let tabs = &frame.panes[0].strip.tabs;
    assert!(!tabs[0].active);
    assert!(painted_at(&scene, tabs[0].rect, theme.swatch(Swatch::Blue)));
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
    assert!(painted_at(&scene, tab.rect, theme.swatch(Swatch::Green)));
}

#[test]
fn a_plain_tab_shows_no_line_until_it_is_open() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let frame = frame(&mut text, &coloured(None, None));
    let scene = paint(&mut text, &frame, &theme);
    let tabs = &frame.panes[0].strip.tabs;
    assert!(!painted_at(&scene, tabs[0].rect, theme.accent));
    assert!(painted_at(&scene, tabs[1].rect, theme.accent));
}
