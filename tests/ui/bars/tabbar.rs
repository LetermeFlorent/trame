use trame::geometry::Rect;
use trame::render::Scene;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::{Appearance, Rgba, Swatch, Theme};
use trame::ui::chrome::draw_chrome;
use trame::ui::frame::SidebarItem;
use trame::ui::{Density, FrameInput, FrameLayout, Motion, PanelSide, UiMetrics};
use trame::workspace::Workspace;

use super::painted_at;
use crate::paint::paint_with;
use crate::{frame, frame_with_metrics, input, text_system, workspace};

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

fn line_over(scene: &Scene, tab: Rect, color: Rgba) -> Option<Rect> {
    scene
        .quads()
        .iter()
        .filter(|quad| quad.color == color.to_array())
        .map(|quad| Rect::new(quad.rect[0], quad.rect[1], quad.rect[2], quad.rect[3]))
        .find(|line| {
            (tab.y..=tab.y + 1.0 + EPSILON).contains(&line.y)
                && line.x >= tab.x
                && line.right() <= tab.right() + EPSILON
        })
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
fn a_plain_tab_is_outlined_only_once_it_is_open() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let frame = frame(&mut text, &coloured(None, None));
    let scene = paint(&mut text, &frame, &theme);
    let tabs = &frame.panes[0].strip.tabs;
    assert!(line_over(&scene, tabs[0].rect, theme.accent).is_none());
    let outline = line_over(&scene, tabs[1].rect, theme.accent).expect("an outline");
    assert_eq!(outline.x, tabs[1].rect.x);
    assert!((outline.right() - tabs[1].rect.right()).abs() < EPSILON);
    assert!(outline.bottom() >= tabs[1].rect.bottom());
}

#[test]
fn a_rounded_tab_keeps_its_colour_line_off_the_corners() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let metrics = UiMetrics::new(1.0).with_display(PanelSide::Left, Density::Spacious);
    let workspace = coloured(Some(Swatch::Blue), Some(Swatch::Green));
    let frame = frame_with_metrics(&mut text, &workspace, metrics);
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
        let frame = frame_with_metrics(&mut text, &workspace, UiMetrics::new(scale));
        let strip = &frame.panes[0].strip;
        for button in [strip.new_tab, strip.split] {
            let above = button.y - strip.rect.y;
            let below = strip.rect.bottom() - button.bottom();
            assert!(above >= 1.0 && below >= 1.0, "{scale}: {above} {below}");
            assert!((above - below).abs() <= 1.0, "{scale}: {above} {below}");
        }
    }
}

#[test]
fn a_working_tab_marks_its_own_tab_and_its_instance() {
    let mut text = text_system();
    let workspace = workspace();
    let mut frame = frame(&mut text, &workspace);
    assert!(!frame.shows_work());
    frame.mark_working(&workspace, |id| id == SessionId(1));
    let tabs = &frame.panes[0].strip.tabs;
    assert!(tabs[0].working && !tabs[1].working);
    let sidebar = frame.sidebar.as_ref().expect("a sidebar");
    let instance = SidebarItem::Instance(workspace.instances[0].id);
    let row = sidebar.rows.iter().find(|row| row.item == instance);
    assert!(row.expect("the instance row").working);
    assert!(frame.shows_work());
}

#[test]
fn a_working_tab_weaves_the_logo_in_place_of_its_close_button() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let workspace = workspace();
    let mut frame = frame(&mut text, &workspace);
    let close = frame.panes[0].strip.tabs[0].close;
    let woven = |scene: &Scene| {
        scene.quads().iter().any(|quad| {
            let rect = Rect::new(quad.rect[0], quad.rect[1], quad.rect[2], quad.rect[3]);
            quad.color[..3] == Rgba::WHITE.to_array()[..3] && !rect.intersect(close).is_empty()
        })
    };
    assert!(!woven(&paint(&mut text, &frame, &theme)));
    frame.mark_working(&workspace, |id| id == SessionId(1));
    assert!(woven(&paint(&mut text, &frame, &theme)));
}

#[test]
fn even_tabs_take_the_width_of_the_longest_name_until_toggled_back() {
    let mut text = text_system();
    let mut workspace = workspace();
    let title = |id: SessionId| match id.0 {
        2 => "a much longer tab name".to_owned(),
        _ => "pwsh".to_owned(),
    };
    let widths = |workspace: &Workspace, text: &mut TextSystem| {
        let input = FrameInput {
            title: &title,
            ..input(workspace, UiMetrics::new(1.0))
        };
        let frame = FrameLayout::compute_with(text, &input, &mut Motion::default());
        let tabs = &frame.panes[0].strip.tabs;
        (tabs[0].rect.width, tabs[1].rect.width)
    };
    let (short, long) = widths(&workspace, &mut text);
    assert!(short < long);
    workspace.toggle_even_tabs(SessionId(1));
    assert_eq!(widths(&workspace, &mut text), (long, long));
    workspace.toggle_even_tabs(SessionId(2));
    assert_eq!(widths(&workspace, &mut text), (short, long));
}
