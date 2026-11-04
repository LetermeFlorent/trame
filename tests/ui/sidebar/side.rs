use trame::geometry::{Point, Rect};
use trame::theme::{Appearance, Theme};
use trame::ui::chrome::draw_chrome;
use trame::ui::{Density, FrameLayout, Hit, PanelSide, UiMetrics};

use crate::paint::paint_with;
use crate::{WIDTH, frame_in, text_system, workspace};

fn on(panel: PanelSide, density: Density) -> UiMetrics {
    UiMetrics::new(1.0).with_display(panel, density)
}

fn row_height(frame: &FrameLayout) -> f32 {
    frame.sidebar.as_ref().expect("the panel is open").rows[0]
        .rect
        .height
}

#[test]
fn a_right_panel_sits_after_the_terminals() {
    let mut text = text_system();
    let frame = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Right, Density::Compact),
    );
    assert_eq!(frame.side.right(), WIDTH);
    assert_eq!(frame.content.x, 0.0);
    assert!(frame.content.right() <= frame.side.x);
    let sidebar = frame.sidebar.as_ref().expect("the panel is open");
    assert_eq!(sidebar.rect.x, frame.side.x);
    let edge = Point::new(frame.side.x, frame.side.center().y);
    assert_eq!(frame.hit(edge, false), Some(Hit::SidebarEdge));
}

#[test]
fn both_sides_leave_the_terminals_the_same_room() {
    let mut text = text_system();
    let left = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Left, Density::Compact),
    );
    let right = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Right, Density::Compact),
    );
    assert_eq!(left.side.width, right.side.width);
    assert_eq!(left.content.width, right.content.width);
    assert_eq!(left.content.x, right.side.width);
}

#[test]
fn spacious_sets_the_terminal_in_a_rounded_card() {
    let mut text = text_system();
    let compact = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Left, Density::Compact),
    );
    let spacious = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Left, Density::Spacious),
    );
    assert_eq!(row_height(&spacious), row_height(&compact));
    let (flat, card) = (&compact.panes[0], &spacious.panes[0]);
    assert_eq!(flat.radius, 0.0);
    assert_eq!(card.radius, UiMetrics::CARD_RADIUS);
    assert_eq!(
        card.strip.rect.y,
        flat.strip.rect.y + UiMetrics::CARD_MARGIN
    );
    assert_eq!(card.body.x, flat.body.x + UiMetrics::CARD_MARGIN);
    assert_eq!(
        card.body.bottom(),
        flat.body.bottom() - UiMetrics::CARD_MARGIN
    );
}

#[test]
fn compact_keeps_the_roomy_sizes() {
    let mut text = text_system();
    let frame = frame_in(
        &mut text,
        &workspace(),
        on(PanelSide::Left, Density::Compact),
    );
    assert_eq!(row_height(&frame), (UiMetrics::SIDEBAR_ROW * 1.25).round());
    assert_eq!(
        frame.panes[0].strip.rect.height,
        (UiMetrics::TAB_BAR * 1.25).round()
    );
}

fn covers(quad: [f32; 4], rect: Rect) -> bool {
    let [x, y, width, height] = quad;
    x <= rect.x && y <= rect.y && x + width >= rect.right() && y + height >= rect.bottom()
}

fn beside(frame: &FrameLayout) -> Rect {
    let side = frame.side;
    match frame.metrics.panel {
        PanelSide::Left => Rect::from_edges(side.right(), side.y, WIDTH, side.bottom()),
        PanelSide::Right => Rect::from_edges(0.0, side.y, side.x, side.bottom()),
    }
}

#[test]
fn over_the_backdrop_only_the_panel_is_see_through() {
    for panel in PanelSide::ALL {
        let mut text = text_system();
        let mut frame = frame_in(&mut text, &workspace(), on(panel, Density::Spacious));
        frame.panes.clear();
        for backdrop in [false, true] {
            let theme = Theme {
                backdrop,
                ..Theme::new(Appearance::Dark)
            };
            let scene = paint_with(&mut text, &frame, &theme, None, draw_chrome);
            let painted = |rect: Rect, color: [f32; 4]| {
                let quads = scene.quads().iter();
                quads
                    .filter(|quad| quad.color == color)
                    .any(|quad| covers(quad.rect, rect))
            };
            assert!(painted(frame.side, theme.panel().to_array()));
            let chrome = theme.chrome.to_array();
            assert_eq!(painted(beside(&frame), chrome), backdrop, "{panel:?}");
            assert!(painted(frame.title_bar.rect, chrome));
        }
    }
}
