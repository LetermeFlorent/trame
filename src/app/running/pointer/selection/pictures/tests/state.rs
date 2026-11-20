use std::sync::Arc;
use std::time::Instant;

use trame::files::encode_png;
use trame::files::pictures::fit_size;
use trame::geometry::Rect;
use trame::input::MouseButton;
use trame::session::SessionId;
use trame::text::{DEFAULT_LINE_HEIGHT, DecodedPicture, TextSystem};
use trame::ui::UiMetrics;
use trame::ui::chrome::{PictureFrame, place_picture};

use super::super::super::Selecting;
use super::super::hover::hover_preview_allowed;
use super::super::state::{PictureView, picture_target, sync_picture_job};

#[test]
fn picture_frame_updates_from_bounds_using_uploaded_dims_without_reupload() {
    let metrics = UiMetrics::new(1.0);
    let mut view = PictureView::new(1, Arc::from(&[][..]), (200, 400));
    view.uploaded = Some((200, 400));
    view.update_frame(Rect::new(0.0, 0.0, 800.0, 600.0), 32.0, &metrics, None);
    assert_eq!(
        view.frame,
        PictureFrame::centered(Rect::new(0.0, 0.0, 800.0, 600.0), 200, 400)
    );
    view.update_frame(Rect::new(0.0, 0.0, 1200.0, 600.0), 32.0, &metrics, None);
    assert_eq!(
        view.frame,
        PictureFrame::centered(Rect::new(0.0, 0.0, 1200.0, 600.0), 200, 400)
    );
}

#[test]
fn picture_target_caps_to_area_and_max_texture_without_double_quantization() {
    let metrics = UiMetrics::new(1.0);
    let bounds = Rect::new(0.0, 0.0, 1180.0, 740.0);
    let target = picture_target(bounds, &metrics, Some((4000, 3000)), 16384);
    assert!(target.0 <= 1180 && target.1 <= 740);
    let capped = picture_target(
        Rect::new(0.0, 0.0, 3000.0, 3000.0),
        &metrics,
        Some((4000, 3000)),
        1024,
    );
    assert!(capped.0 <= 1024 && capped.1 <= 1024);

    let small_bounds = Rect::new(0.0, 0.0, 365.0, 365.0);
    let src = (1000, 333);
    let limit = picture_target(small_bounds, &metrics, Some(src), 16384);
    let placed = place_picture(small_bounds, &metrics, src.0, src.1);
    assert_eq!(fit_size(src.0, src.1, limit.0, limit.1), placed.size);
}

#[test]
fn picture_sync_cancels_stale_resize_job_when_target_snaps_back_to_ready() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    let png = Arc::<[u8]>::from(encode_png(4, 4, &[255; 64]).expect("png"));
    let mut view = PictureView::new(1, png, (1000, 1000));
    view.ready = Some(DecodedPicture {
        width: 400,
        height: 400,
        src_size: (1000, 1000),
        pixels: vec![0; 400 * 400 * 4],
    });
    sync_picture_job(&mut view, &mut text, (200, 200));
    assert!(text.has_picture_job(1));
    assert_eq!(view.requested, Some((200, 200)));

    sync_picture_job(&mut view, &mut text, (400, 400));
    assert!(!text.has_picture_job(1));
    assert_eq!(view.requested, None);
}

#[test]
fn picture_upload_skips_ready_larger_than_max_tex() {
    let mut view = PictureView::new(1, Arc::from(&[][..]), (2048, 1024));
    view.ready = Some(DecodedPicture {
        width: 2048,
        height: 1024,
        src_size: (2048, 1024),
        pixels: vec![0; 2048 * 1024 * 4],
    });
    assert!(view.pending_upload(1024).is_none());
    assert!(view.pending_upload(2048).is_some());
}

#[test]
fn picture_selecting_clear_resets_all_fields() {
    let mut selecting = Selecting {
        drag: Some(SessionId(1)),
        held: Some((SessionId(1), MouseButton::Left)),
        cell: Some((2, 3)),
        clicks: Some((Instant::now(), (2, 3), 2)),
    };
    assert!(!selecting.is_clear());
    selecting.clear();
    assert!(selecting.is_clear());
}

#[test]
fn hover_preview_only_triggers_when_primary_held_and_unblocked() {
    assert!(hover_preview_allowed(false, false, true));
    assert!(!hover_preview_allowed(false, false, false));
    assert!(!hover_preview_allowed(true, false, true));
    assert!(!hover_preview_allowed(false, true, true));
}
