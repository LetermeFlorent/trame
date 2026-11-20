use std::path::PathBuf;
use std::sync::Arc;

use trame::files::pictures::fit_size;
use trame::geometry::{Point, Rect};
use trame::text::{DecodedPicture, ImageSource, TextSystem};
use trame::ui::UiMetrics;
use trame::ui::chrome::{PictureFrame, picture_area_limit, place_picture_open};

use super::actions::picture_hover_hit;

pub(in crate::app::running) struct PictureView {
    pub(super) job: u64,
    pub(super) source: ImageSource,
    pub(super) path: Option<PathBuf>,
    pub(super) src_size: Option<(u32, u32)>,
    pub(super) frame: PictureFrame,
    pub(super) requested: Option<(u32, u32)>,
    pub(super) ready: Option<DecodedPicture>,
    pub(in crate::app::running) uploaded: Option<(u32, u32)>,
}

impl PictureView {
    pub(in crate::app::running) fn new(job: u64, bytes: Arc<[u8]>, src_size: (u32, u32)) -> Self {
        Self {
            job,
            source: ImageSource::Bytes(bytes),
            path: None,
            src_size: Some(src_size),
            frame: PictureFrame::default(),
            requested: None,
            ready: None,
            uploaded: None,
        }
    }

    pub(in crate::app::running) fn from_path(job: u64, path: PathBuf) -> Self {
        Self {
            job,
            source: ImageSource::Path(Arc::new(path.clone())),
            path: Some(path),
            src_size: None,
            frame: PictureFrame::default(),
            requested: None,
            ready: None,
            uploaded: None,
        }
    }

    pub(super) fn needs_request(&self, limit: (u32, u32)) -> bool {
        self.requested != Some(limit) && !self.matches_ready(limit)
    }

    pub(super) fn matches_ready(&self, limit: (u32, u32)) -> bool {
        let (Some(src), Some(ready)) = (self.src_size, &self.ready) else {
            return false;
        };
        let target = fit_size(src.0, src.1, limit.0, limit.1);
        (ready.width, ready.height) == target
    }

    pub(super) fn pending_upload(&self, max_tex: u32) -> Option<(u32, u32, &[u8])> {
        let ready = self.ready.as_ref()?;
        (ready.width <= max_tex
            && ready.height <= max_tex
            && self.uploaded != Some((ready.width, ready.height)))
        .then_some((ready.width, ready.height, ready.pixels.as_slice()))
    }

    pub(super) fn update_frame(
        &mut self,
        bounds: Rect,
        title_bottom: f32,
        metrics: &UiMetrics,
        text: Option<(&mut TextSystem, &'static str)>,
    ) {
        let hovered = self.frame.hovered;
        let mut frame = self.uploaded.map_or_else(PictureFrame::default, |(w, h)| {
            PictureFrame::centered(bounds, w, h)
        });
        if self.path.is_some()
            && let Some((text, label)) = text
        {
            frame.open = place_picture_open(text, bounds, title_bottom, metrics, label);
            frame.open_label = frame.open.map(|_| label);
        }
        frame.hovered = hovered;
        self.frame = frame;
    }

    pub(super) fn sync_hover(&mut self, close: Rect, position: Option<Point>) -> bool {
        let hovered = picture_hover_hit(close, self.frame.open, position);
        let changed = self.frame.hovered != hovered;
        self.frame.hovered = hovered;
        changed
    }
}

pub(super) fn sync_picture_job(
    picture: &mut PictureView,
    text: &mut TextSystem,
    target: (u32, u32),
) {
    if picture.needs_request(target) {
        picture.requested = Some(target);
        text.request_picture(picture.job, target, picture.source.clone());
    } else if picture.matches_ready(target) && picture.requested.take().is_some() {
        text.cancel_picture();
    }
}

pub(super) fn picture_target(
    bounds: Rect,
    metrics: &UiMetrics,
    src: Option<(u32, u32)>,
    max_tex: u32,
) -> (u32, u32) {
    if src.is_some_and(|(w, h)| w == 0 || h == 0) {
        return (0, 0);
    }
    let (max_w, max_h) = picture_area_limit(bounds, metrics);
    (max_w.min(max_tex), max_h.min(max_tex))
}
