use std::sync::Arc;

use trame::files::encode_png;
use trame::geometry::Rect;
use trame::term::{Cell, Color, Flags, Modes};
use trame::text::ImageSource;
use trame::ui::terminal::links::{LinkKind, LinkSpan};

use super::rules::{
    KeyOutcome, grid_cell, inline_click_opens, link_active, overlay_key, reports_mouse, span_anchor,
};
use crate::gnome::view::paint::strips::{RowCells, image_strips, strips_bounds};
use crate::gnome::worker::{PreviewKey, WorkerAnswer, WorkerJob, decode_job};

fn image_cell(id: u32) -> Cell {
    Cell {
        ch: '\u{100000}',
        fg: Color::Rgb(
            ((id >> 16) & 0xFF) as u8,
            ((id >> 8) & 0xFF) as u8,
            (id & 0xFF) as u8,
        ),
        bg: Color::Background,
        flags: Flags::IMAGE,
        link: 0,
    }
}

fn sample_png(width: u32, height: u32) -> Vec<u8> {
    let pixels = vec![200u8; (width * height * 4) as usize];
    encode_png(width as usize, height as usize, &pixels).expect("valid png")
}

#[test]
fn inline_and_overlay_rules_match_classic_behavior() {
    assert!(!reports_mouse(Modes::default(), false));
    assert!(reports_mouse(Modes::MOUSE_MOTION, false));
    assert!(!reports_mouse(Modes::MOUSE_MOTION, true));
    assert!(inline_click_opens(false, false));
    assert!(!inline_click_opens(true, false));
    assert!(inline_click_opens(true, true));
    assert!(link_active(false, false, true));
    assert!(!link_active(true, false, true));
    assert!(!link_active(false, true, true));
    assert_eq!(overlay_key(false, true), KeyOutcome::Pass);
    assert_eq!(overlay_key(true, true), KeyOutcome::Close);
    assert_eq!(overlay_key(true, false), KeyOutcome::Consume);
}

#[test]
fn grid_cell_and_span_anchor_compute_exact_bounds() {
    assert_eq!(
        grid_cell((25.0, 41.0), (10.0, 20.0), (24, 80)),
        Some((2, 2))
    );
    assert_eq!(grid_cell((-1.0, 5.0), (10.0, 20.0), (24, 80)), None);
    assert_eq!(grid_cell((800.0, 5.0), (10.0, 20.0), (24, 80)), None);
    let span = LinkSpan {
        row: 3,
        start: 4,
        end: 10,
        target: "sample.png".into(),
        kind: LinkKind::Image,
    };
    assert_eq!(
        span_anchor(&span, (8.0, 16.0)),
        Rect::new(32.0, 48.0, 48.0, 16.0)
    );
}

#[test]
fn image_strips_split_around_overwritten_text_and_merge_rows() {
    let first = [image_cell(7), image_cell(7), image_cell(7)];
    let second = [image_cell(7), Cell::default(), image_cell(7)];
    let third = [image_cell(7), Cell::default(), image_cell(7)];
    let rows = [
        RowCells {
            row: 0,
            y: 0.0,
            cells: &first,
        },
        RowCells {
            row: 1,
            y: 20.0,
            cells: &second,
        },
        RowCells {
            row: 2,
            y: 40.0,
            cells: &third,
        },
    ];
    let strips = image_strips(&rows, 7, (10.0, 20.0));
    assert_eq!(
        strips,
        vec![
            Rect::new(0.0, 0.0, 30.0, 20.0),
            Rect::new(0.0, 20.0, 10.0, 20.0),
            Rect::new(20.0, 20.0, 10.0, 20.0),
            Rect::new(0.0, 40.0, 10.0, 20.0),
            Rect::new(20.0, 40.0, 10.0, 20.0),
        ]
    );
    assert_eq!(strips_bounds(&strips), Rect::new(0.0, 0.0, 30.0, 60.0));
}

#[test]
fn worker_decodes_inline_overlay_hover_and_rejects_corrupt_bytes() {
    let bytes: Arc<[u8]> = sample_png(40, 20).into();
    let key = PreviewKey {
        session_id: 1,
        image_id: 2,
        width: 20,
        height: 10,
    };
    let inline = decode_job(WorkerJob::Inline(key, (20, 20), Arc::clone(&bytes)));
    assert!(matches!(
        inline,
        WorkerAnswer::Inline(_, Some(ref r)) if (r.width, r.height) == (20, 10)
    ));
    let broken = decode_job(WorkerJob::Inline(key, (20, 20), Arc::from(&b"bad"[..])));
    assert!(matches!(broken, WorkerAnswer::Inline(_, None)));
    let overlay = decode_job(WorkerJob::Overlay(9, (500, 500), ImageSource::Bytes(bytes)));
    assert!(matches!(
        overlay,
        WorkerAnswer::Overlay(9, Some(ref r)) if (r.width, r.height) == (40, 20)
    ));
}
