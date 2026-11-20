mod actions;
mod hover;
mod input;
mod state;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use trame::files::pictures::open_image_file;
use trame::session::SessionId;
use trame::ui::chrome::PictureFrame;

use super::{Cell, Running};
use state::{picture_target, sync_picture_job};

pub(in crate::app::running::pointer::selection) use actions::{PictureAction, picture_action};
pub(in crate::app::running) use hover::HoverPreview;
pub(in crate::app::running) use state::PictureView;

static NEXT_PICTURE_JOB: AtomicU64 = AtomicU64::new(1);

impl Running {
    fn show_picture(&mut self, view: PictureView) {
        self.selecting.clear();
        self.clear_hover_preview();
        self.picture = Some(view);
        self.sync_picture();
        self.changed();
    }

    pub(super) fn open_picture_at(&mut self, id: SessionId, cell: Cell) -> bool {
        let stored = self
            .desk
            .sessions
            .get(id)
            .and_then(|session| session.terminal().screen().image_at(cell.0, cell.1));
        let Some((_, stored)) = stored else {
            return false;
        };
        let job = NEXT_PICTURE_JOB.fetch_add(1, Ordering::Relaxed);
        self.show_picture(PictureView::new(
            job,
            Arc::clone(&stored.bytes),
            (stored.width, stored.height),
        ));
        true
    }

    pub(super) fn open_picture_path(&mut self, path: PathBuf) {
        let job = NEXT_PICTURE_JOB.fetch_add(1, Ordering::Relaxed);
        self.show_picture(PictureView::from_path(job, path));
    }

    pub(super) fn open_picture_external(&self) {
        if let Some(path) = self.picture.as_ref().and_then(|p| p.path.as_ref()) {
            open_image_file(path.clone());
        }
    }

    fn receive_picture(&mut self, job: u64) -> bool {
        match self.text.take_picture(job) {
            Some(Some(decoded)) => {
                if let Some(picture) = &mut self.picture {
                    picture.src_size = Some(decoded.src_size);
                    picture.ready = Some(decoded);
                    picture.requested = None;
                }
                true
            }
            Some(None) => {
                self.close_picture();
                false
            }
            None => true,
        }
    }

    pub(in crate::app::running) fn sync_picture(&mut self) {
        self.sync_hover_preview();
        let Some(job) = self.picture.as_ref().map(|p| p.job) else {
            return;
        };
        if !self.receive_picture(job) {
            return;
        }
        let (bounds, max_tex) = (self.frame.bounds, self.graphics.max_texture_size());
        let Some(picture) = &mut self.picture else {
            return;
        };
        let target = picture_target(bounds, &self.metrics, picture.src_size, max_tex);
        if target.0 == 0 || target.1 == 0 {
            picture.frame = PictureFrame::default();
            return;
        }
        sync_picture_job(picture, &mut self.text, target);
        if let Some((width, height, pixels)) = picture.pending_upload(max_tex) {
            self.graphics.set_picture(Some((width, height, pixels)));
            picture.uploaded = Some((width, height));
        }
        self.place_picture();
    }

    pub(in crate::app::running) fn place_picture(&mut self) {
        let (bounds, title_bottom, max_tex) = (
            self.frame.bounds,
            self.frame.title_bar.rect.bottom(),
            self.graphics.max_texture_size(),
        );
        let Some(picture) = &mut self.picture else {
            return;
        };
        let target = picture_target(bounds, &self.metrics, picture.src_size, max_tex);
        if target.0 == 0 || target.1 == 0 {
            picture.frame = PictureFrame::default();
            return;
        }
        picture.update_frame(
            bounds,
            title_bottom,
            &self.metrics,
            Some((&mut self.text, self.desk.strings.open_in_viewer)),
        );
    }

    pub(in crate::app::running) fn picture_frame(&self) -> Option<PictureFrame> {
        self.picture.as_ref().map(|view| view.frame)
    }

    pub fn picture_ready(&mut self) {
        self.window.request_redraw();
    }

    pub(in crate::app::running) fn close_picture(&mut self) -> bool {
        if self.picture.take().is_none() {
            return false;
        }
        self.text.cancel_picture();
        self.graphics.set_picture(None);
        self.selecting.clear();
        self.drop_press();
        self.changed();
        true
    }
}
