mod bar;
mod motion;

use trame::geometry::Point;
use trame::render::Scene;
use trame::text::TextSystem;
use trame::theme::{Appearance, Theme};
use trame::ui::chrome::{LoaderView, draw_loader};
use trame::ui::{FrameLayout, Hit, ResizeEdge};

use crate::paint::paint_with;
use crate::{HEIGHT, WIDTH, frame, text_system, workspace};

const DOTS: usize = 10;
const LOGO_STRANDS: usize = 16;

fn waiting(opacity: f32) -> LoaderView<'static> {
    LoaderView {
        message: "Checking for updates",
        previous: None,
        blend: 1.0,
        progress: None,
        spin: 0.3,
        opacity,
        content: 1.0,
        shift: 0.0,
    }
}

fn paint(text: &mut TextSystem, layout: &FrameLayout, theme: &Theme, loader: &LoaderView) -> Scene {
    paint_with(text, layout, theme, None, |scene, text, input| {
        draw_loader(scene, text, input, loader)
    })
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
fn the_loader_hides_the_title_and_the_sidebar_button_but_not_the_window_buttons() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &workspace());
    let scene = paint(&mut text, &layout, &theme, &waiting(1.0));
    let bar = layout.title_bar.rect;
    let cover = scene.quads()[1].rect;
    assert_eq!(scene.quads()[1].color, theme.chrome.to_array());
    assert_eq!([cover[0], cover[1]], [bar.x, bar.y]);
    assert!(cover[3] < bar.height);
    assert!(cover[0] + cover[2] >= layout.title_bar.toggle.right());
    if let Some(controls) = layout.title_bar.controls {
        assert!(cover[0] + cover[2] <= controls.minimize.x);
    }
}

#[test]
fn the_logo_ring_and_message_are_centered() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &workspace());
    let scene = paint(&mut text, &layout, &theme, &waiting(1.0));
    let content = &scene.quads()[2..];
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
    let empty = layout.title_bar.rect.center();
    assert_eq!(layout.hit(empty, false), Some(Hit::TitleBar));
}

#[test]
fn every_window_border_resizes_toward_its_own_side() {
    let mut text = text_system();
    let layout = frame(&mut text, &workspace());
    let (right, bottom) = (WIDTH - 1.0, HEIGHT - 1.0);
    let (middle_x, middle_y) = (WIDTH / 2.0, HEIGHT / 2.0);
    let cases = [
        (Point::new(middle_x, 1.0), ResizeEdge::North),
        (Point::new(middle_x, bottom), ResizeEdge::South),
        (Point::new(1.0, middle_y), ResizeEdge::West),
        (Point::new(right, middle_y), ResizeEdge::East),
        (Point::new(right, 1.0), ResizeEdge::NorthEast),
        (Point::new(1.0, bottom), ResizeEdge::SouthWest),
        (Point::new(right, bottom), ResizeEdge::SouthEast),
    ];
    for (point, edge) in cases {
        assert_eq!(layout.hit(point, true), Some(Hit::Resize(edge)));
    }
}
