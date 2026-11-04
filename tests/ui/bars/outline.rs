use trame::geometry::Rect;
use trame::preferences::SshHost;
use trame::render::Scene;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::{Appearance, Swatch, Theme};
use trame::ui::chrome::draw_chrome;
use trame::ui::{Density, PanelSide, UiMetrics};
use trame::workspace::Workspace;

use crate::paint::paint_with;
use crate::{frame_with_metrics, text_system, workspace};

fn with_host(tab: SessionId) -> Workspace {
    let mut workspace = workspace();
    let host = SshHost::parse("me@box").expect("a host");
    workspace.set_tab_host(tab, Some(host));
    workspace
}

fn outlined(scene: &Scene, tab: Rect, theme: &Theme) -> bool {
    scene.quads().iter().any(|quad| {
        quad.color == theme.tab_outline.to_array() && quad.rect[..3] == [tab.x, tab.y, tab.width]
    })
}

#[test]
fn an_ssh_tab_is_outlined_and_a_plain_tab_is_not() {
    for appearance in [Appearance::Dark, Appearance::Light] {
        for density in [Density::Compact, Density::Spacious] {
            let mut text = text_system();
            let theme = Theme::new(appearance);
            let metrics = UiMetrics::new(1.0).with_display(PanelSide::Left, density);
            assert_only_ssh_outlined(&mut text, &theme, metrics);
        }
    }
}

fn assert_only_ssh_outlined(text: &mut TextSystem, theme: &Theme, metrics: UiMetrics) {
    for ssh in [SessionId(1), SessionId(2)] {
        let workspace = with_host(ssh);
        let frame = frame_with_metrics(text, &workspace, metrics);
        let scene = paint_with(text, &frame, theme, None, draw_chrome);
        for tab in &frame.panes[0].strip.tabs {
            assert_eq!(outlined(&scene, tab.rect, theme), tab.id == ssh);
        }
    }
}

#[test]
fn the_outline_keeps_the_label_and_the_colour_line() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let metrics = UiMetrics::new(1.0);
    let mut workspace = with_host(SessionId(1));
    workspace.set_tab_color(SessionId(1), Some(Swatch::Blue));
    let frame = frame_with_metrics(&mut text, &workspace, metrics);
    let plain = frame_with_metrics(&mut text, &crate::workspace(), metrics);
    let tab = &frame.panes[0].strip.tabs[0];
    assert_eq!(tab.label_x, plain.panes[0].strip.tabs[0].label_x);
    let scene = paint_with(&mut text, &frame, &theme, None, draw_chrome);
    assert!(outlined(&scene, tab.rect, &theme));
    let blue = theme.swatch(Swatch::Blue).to_array();
    assert!(
        scene
            .quads()
            .iter()
            .any(|quad| quad.color == blue && quad.rect[1] == tab.rect.y)
    );
}

#[test]
fn each_theme_has_its_own_outline_colour() {
    let dark = Theme::new(Appearance::Dark).tab_outline;
    let light = Theme::new(Appearance::Light).tab_outline;
    assert_ne!(dark, light);
}
