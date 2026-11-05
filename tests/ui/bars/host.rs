use trame::preferences::SshHost;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::{Appearance, Theme};
use trame::ui::chrome::draw_chrome;
use trame::ui::{FrameLayout, Hit, UiMetrics};
use trame::workspace::Workspace;

use crate::paint::paint_with;
use crate::{closed_sidebar, frame, input, text_system, workspace};

const HOST: &str = "root@10.0.0.2";
const LONG_HOST: &str = "administrator@build-server.internal.example.org";
const NARROW: f32 = 400.0;
const TINY: f32 = 140.0;

fn with_host(mut workspace: Workspace, tab: SessionId, target: &str) -> Workspace {
    let host = SshHost::parse(target).expect("a host");
    workspace.set_tab_host(tab, Some(host));
    workspace
}

fn frame_at_width(text: &mut TextSystem, workspace: &Workspace, width: f32) -> FrameLayout {
    let mut input = input(workspace, UiMetrics::new(1.0));
    input.width = width;
    FrameLayout::compute(text, &input)
}

#[test]
fn the_badge_shows_the_host_only_while_its_tab_is_active() {
    let mut text = text_system();
    let plain = frame(&mut text, &workspace());
    let active = plain.panes[0].active.expect("an active tab");
    for tab in [SessionId(1), SessionId(2)] {
        let layout = frame(&mut text, &with_host(workspace(), tab, HOST));
        let strip = &layout.panes[0].strip;
        match tab == active {
            true => {
                let badge = strip.host.as_ref().expect("a badge");
                let label = badge.label.as_ref().expect("a label");
                let full = text.layout(HOST, layout.metrics.label()).width;
                assert_eq!(label.width, full);
                assert!(strip.tabs_area.right() < badge.rect.x);
                assert!(badge.rect.right() < strip.new_tab.x);
            }
            false => {
                assert!(strip.host.is_none());
                assert_eq!(strip.tabs_area, plain.panes[0].strip.tabs_area);
            }
        }
    }
}

#[test]
fn a_click_on_the_badge_lands_on_the_empty_tab_bar() {
    let mut text = text_system();
    let active = frame(&mut text, &workspace()).panes[0]
        .active
        .expect("a tab");
    let layout = frame(&mut text, &with_host(workspace(), active, HOST));
    let pane = &layout.panes[0];
    let badge = pane.strip.host.as_ref().expect("a badge");
    for point in [badge.icon.center(), badge.rect.center()] {
        assert_eq!(layout.hit(point, false), Some(Hit::TabBar(pane.id)));
    }
    for appearance in [Appearance::Dark, Appearance::Light] {
        let theme = Theme::new(appearance);
        let hover = Some(Hit::TabBar(pane.id));
        paint_with(&mut text, &layout, &theme, hover, draw_chrome);
    }
}

#[test]
fn a_narrow_pane_cuts_the_label_then_keeps_only_the_globe() {
    let mut text = text_system();
    let base = closed_sidebar();
    let active = frame(&mut text, &base).panes[0].active.expect("a tab");
    let workspace = with_host(base, active, LONG_HOST);
    let narrow = frame_at_width(&mut text, &workspace, NARROW);
    let strip = &narrow.panes[0].strip;
    let badge = strip.host.as_ref().expect("a badge");
    let full = text.layout(LONG_HOST, narrow.metrics.label()).width;
    let cut = badge.label.as_ref().expect("a cut label");
    assert!(cut.width < full);
    assert!(badge.label_x + cut.width <= badge.rect.right());
    let tiny = frame_at_width(&mut text, &workspace, TINY);
    let badge = tiny.panes[0].strip.host.as_ref().expect("a globe");
    assert!(badge.label.is_none());
    assert_eq!(badge.rect.width, badge.icon.width);
}
