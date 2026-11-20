use std::path::PathBuf;

use trame::geometry::{Point, Rect};
use trame::text::{CellMetrics, DEFAULT_LINE_HEIGHT, DecodedPicture, TextSystem};
use trame::ui::chrome::{PictureButton, picture_close_rect};
use trame::ui::terminal::links::{LinkHit, LinkKind, LinkSpan};
use trame::ui::{Hit, UiMetrics};
use trame::workspace::PaneId;
use winit::window::CursorIcon;

use super::super::actions::picture_cursor;
use super::super::hover::HoverPreview;
use super::super::state::PictureView;
use crate::app::running::pointer::cursor::hover_cursor_for;
use crate::app::running::pointer::selection::links::{LinkHover, link_hover_from_hit};

#[test]
fn picture_hover_resyncs_open_button_after_resize_and_scale() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    let mut view = PictureView::from_path(1, PathBuf::from("shot.png"));
    let bounds1 = Rect::new(0.0, 0.0, 800.0, 600.0);
    let m1 = UiMetrics::new(1.0);
    view.update_frame(bounds1, 32.0, &m1, Some((&mut text, "Open")));
    let old_open = view.frame.open.expect("open button");
    let pointer = Point::new(old_open.x + 4.0, old_open.y + 4.0);
    let close1 = picture_close_rect(bounds1, 32.0, &m1);
    assert!(view.sync_hover(close1, Some(pointer)));
    assert_eq!(view.frame.hovered, Some(PictureButton::Open));

    let bounds2 = Rect::new(0.0, 0.0, 1200.0, 600.0);
    let m2 = UiMetrics::new(2.0);
    view.update_frame(bounds2, 64.0, &m2, Some((&mut text, "Open")));
    let close2 = picture_close_rect(bounds2, 64.0, &m2);
    assert!(view.sync_hover(close2, Some(pointer)));
    assert_eq!(view.frame.hovered, None);
    assert_eq!(
        picture_cursor(close2, view.frame.open, Some(pointer)),
        CursorIcon::Default
    );
    let new_open = view.frame.open.expect("moved open button");
    let on_new = Point::new(new_open.x + 4.0, new_open.y + 4.0);
    assert!(view.sync_hover(close2, Some(on_new)));
    assert_eq!(view.frame.hovered, Some(PictureButton::Open));
    assert_eq!(
        picture_cursor(close2, view.frame.open, Some(on_new)),
        CursorIcon::Pointer
    );
}

#[test]
fn hover_preview_move_anchor_requests_redraw_only_when_ready_and_moved() {
    let a1 = Rect::new(10.0, 20.0, 40.0, 16.0);
    let a2 = Rect::new(10.0, 40.0, 40.0, 16.0);
    let mut preview = HoverPreview {
        path: PathBuf::from("a.png"),
        anchor: a1,
        job: 1,
        limit: (100, 100),
        ready: None,
        failed: false,
    };
    assert!(!preview.move_anchor(a2));
    assert_eq!(preview.anchor, a2);
    preview.ready = Some(DecodedPicture {
        width: 1,
        height: 1,
        src_size: (1, 1),
        pixels: vec![0; 4],
    });
    assert!(!preview.move_anchor(a2));
    assert!(preview.move_anchor(a1));
    assert_eq!(preview.anchor, a1);
}

fn test_cell_metrics() -> CellMetrics {
    CellMetrics {
        font_size: 14.0,
        width: 10.0,
        height: 20.0,
        baseline: 15.0,
        underline_position: 17.0,
        underline_thickness: 1.0,
        strike_position: 10.0,
        strike_thickness: 1.0,
    }
}

#[test]
fn shared_link_hover_drives_preview_and_cursor_from_single_hit() {
    let metrics = test_cell_metrics();
    let terminal = Rect::new(100.0, 50.0, 400.0, 200.0);
    let span = LinkSpan {
        row: 2,
        start: 3,
        end: 8,
        target: "a.png".to_owned(),
        kind: LinkKind::Image,
    };
    let img_hit = LinkHit {
        target: "a.png".to_owned(),
        kind: LinkKind::Image,
        spans: vec![span],
    };
    let hover = link_hover_from_hit(img_hit, terminal, (2, 4), metrics);
    let want = LinkHover::Image(PathBuf::from("a.png"), Rect::new(130.0, 90.0, 50.0, 20.0));
    assert_eq!(hover, want);
    assert_eq!(
        hover_cursor_for(Some(Hit::Terminal(PaneId(1))), &[], true),
        CursorIcon::Pointer
    );
    let web_hit = LinkHit {
        target: "https://example.com".to_owned(),
        kind: LinkKind::Web,
        spans: vec![],
    };
    assert_eq!(
        link_hover_from_hit(web_hit, terminal, (0, 0), metrics),
        LinkHover::Web
    );
}
