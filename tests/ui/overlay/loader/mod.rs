mod bar;

use trame::geometry::Point;
use trame::render::Scene;
use trame::text::TextSystem;
use trame::theme::{Appearance, Theme};
use trame::ui::chrome::{ChromeInput, LoaderView, draw_loader};
use trame::ui::{FrameLayout, FrameView, Hit, Motion, ResizeEdge};

use crate::{HEIGHT, WIDTH, frame, text_system, workspace};

const DOTS: usize = 10;
const LOGO_STRANDS: usize = 16;

fn waiting(opacity: f32) -> LoaderView<'static> {
    LoaderView {
        message: "Checking for updates",
        progress: None,
        spin: 0.3,
        opacity,
    }
}

fn paint(text: &mut TextSystem, layout: &FrameLayout, theme: &Theme, loader: &LoaderView) -> Scene {
    let view = FrameView::default();
    let motion = Motion::default();
    let input = ChromeInput {
        theme,
        frame: layout,
        view: &view,
        motion: &motion,
    };
    let mut scene = Scene::new(layout.bounds.width, layout.bounds.height);
    draw_loader(&mut scene, text, &input, loader);
    scene
}

#[test]
fn the_loader_covers_everything_below_the_title_bar() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &workspace());
    let scene = paint(&mut text, &layout, &theme, &waiting(1.0));
    let bar = layout.title_bar.rect.height;
    let backdrop = scene.quads().first().expect("a backdrop is drawn");
    assert_eq!(backdrop.rect, [0.0, bar, WIDTH, HEIGHT - bar]);
    assert_eq!(backdrop.color, theme.chrome.to_array());
    assert!(scene.quads().len() > 1 + LOGO_STRANDS + DOTS);
}

#[test]
fn the_logo_ring_and_message_are_centered() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &workspace());
    let scene = paint(&mut text, &layout, &theme, &waiting(1.0));
    let content = &scene.quads()[1..];
    let left = content
        .iter()
        .map(|quad| quad.rect[0])
        .fold(f32::MAX, f32::min);
    let right = content
        .iter()
        .map(|quad| quad.rect[0] + quad.rect[2])
        .fold(f32::MIN, f32::max);
    let middle = WIDTH / 2.0;
    assert!((middle - left - (right - middle)).abs() <= 2.0);
}

#[test]
fn a_faded_out_loader_draws_nothing() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &workspace());
    assert!(
        paint(&mut text, &layout, &theme, &waiting(0.0))
            .quads()
            .is_empty()
    );
}

#[test]
fn a_blocked_frame_only_lets_the_window_through() {
    let mut text = text_system();
    let mut layout = frame(&mut text, &workspace());
    let pane = layout.panes[0].body.center();
    let row = layout.sidebar.as_ref().expect("the sidebar").rect.center();
    assert_eq!(
        layout.hit(pane, true),
        Some(Hit::Terminal(layout.panes[0].id))
    );
    layout.blocked = true;
    assert_eq!(layout.hit(pane, true), Some(Hit::Backdrop));
    assert_eq!(layout.hit(row, true), Some(Hit::Backdrop));
    assert_eq!(
        layout.hit(Point::new(1.0, 1.0), true),
        Some(Hit::Resize(ResizeEdge::NorthWest))
    );
    if let Some(controls) = layout.title_bar.controls {
        assert_eq!(layout.hit(controls.close.center(), true), Some(Hit::Close));
        assert_eq!(
            layout.hit(controls.minimize.center(), true),
            Some(Hit::Minimize)
        );
    }
    let bar = layout.title_bar.rect;
    let empty = Point::new(bar.center().x, bar.center().y);
    assert_eq!(layout.hit(empty, false), Some(Hit::TitleBar));
}
