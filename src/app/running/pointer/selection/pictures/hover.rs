use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use trame::geometry::Rect;
use trame::text::DecodedPicture;
use trame::ui::chrome::{HoverPreviewFrame, hover_preview_limit};

use super::Running;

static NEXT_HOVER_JOB: AtomicU64 = AtomicU64::new(1);

pub(in crate::app::running) struct HoverPreview {
    pub(super) path: PathBuf,
    pub(super) anchor: Rect,
    pub(super) job: u64,
    pub(super) limit: (u32, u32),
    pub(super) ready: Option<DecodedPicture>,
    pub(super) failed: bool,
}

pub(super) fn hover_preview_allowed(busy: bool, selecting: bool, primary_held: bool) -> bool {
    !busy && !selecting && primary_held
}

impl Running {
    pub(in crate::app::running) fn refresh_hover_preview(
        &mut self,
        target: Option<(PathBuf, Rect)>,
    ) {
        let overlay = self.menu.is_some()
            || self.editing.is_some()
            || self.settings.is_some()
            || self.setting_field.is_some();
        let busy = self.picture.is_some()
            || self.frame.blocked
            || self.desk.closing.is_some()
            || overlay
            || self.drag.is_some();
        let selecting = self.selecting.drag.is_some() || self.selecting.held.is_some();
        let target = target.filter(|_| hover_preview_allowed(busy, selecting, self.primary_held()));
        match target {
            Some((path, anchor)) => self.set_hover_preview(path, anchor),
            None => self.clear_hover_preview(),
        }
    }

    fn set_hover_preview(&mut self, path: PathBuf, anchor: Rect) {
        let cap = self
            .text
            .max_image_side()
            .min(self.graphics.max_texture_size());
        let limit = hover_preview_limit(&self.metrics, cap);
        if let Some(preview) = &mut self.pointer.preview
            && preview.path == path
        {
            if preview.move_anchor(anchor) {
                self.window.request_redraw();
            }
            if preview.limit != limit && !preview.failed {
                let job = NEXT_HOVER_JOB.fetch_add(1, Ordering::Relaxed);
                preview.job = job;
                preview.limit = limit;
                preview.ready = None;
                self.text.request_preview(job, limit, path);
            }
            return;
        }
        let job = NEXT_HOVER_JOB.fetch_add(1, Ordering::Relaxed);
        self.text.request_preview(job, limit, path.clone());
        self.pointer.preview = Some(HoverPreview {
            path,
            anchor,
            job,
            limit,
            ready: None,
            failed: false,
        });
        self.window.request_redraw();
    }

    pub(in crate::app::running) fn clear_hover_preview(&mut self) {
        if let Some(prev) = self.pointer.preview.take() {
            self.text.cancel_preview();
            if prev.ready.is_some() {
                self.window.request_redraw();
            }
        }
    }

    pub(in crate::app::running) fn sync_hover_preview(&mut self) {
        if self.picture.is_some() || !self.primary_held() {
            return self.clear_hover_preview();
        }
        let Some(preview) = &mut self.pointer.preview else {
            return;
        };
        match self.text.take_preview(preview.job) {
            Some(Some(decoded)) => {
                preview.ready = Some(decoded);
                preview.failed = false;
            }
            Some(None) => {
                preview.ready = None;
                preview.failed = true;
            }
            None => {}
        }
    }
}

impl HoverPreview {
    pub(super) fn move_anchor(&mut self, anchor: Rect) -> bool {
        let redraw = self.anchor != anchor && self.ready.is_some();
        self.anchor = anchor;
        redraw
    }

    pub(in crate::app::running) fn frame(
        picture_open: bool,
        preview: Option<&Self>,
    ) -> Option<HoverPreviewFrame<'_>> {
        if picture_open {
            return None;
        }
        let preview = preview?;
        let ready = preview.ready.as_ref()?;
        Some(HoverPreviewFrame {
            job: preview.job,
            anchor: preview.anchor,
            size: (ready.width, ready.height),
            pixels: ready.pixels.as_slice(),
        })
    }
}
