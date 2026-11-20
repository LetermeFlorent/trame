mod inner;
pub mod rules;
mod style;
#[cfg(test)]
mod tests;
mod widgets;

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};
use trame::log_error;
use trame::text::ImageSource;
use trame::theme::Theme;

use super::view::TermView;
use super::worker::{ReadyPixels, WorkerJob, memory_texture};
use inner::Inner;
use rules::MAX_OVERLAY_SIDE;

#[derive(Clone)]
pub struct PictureOverlay {
    inner: Rc<Inner>,
}

impl PictureOverlay {
    pub fn shared(theme: &Theme) -> Self {
        let ink = Rc::new(Cell::new(theme.text));
        let close_glyph = style::close_icon(Rc::clone(&ink));
        let (root, picture) = widgets::build_scrim();
        let (actions, open_button, close_button) = widgets::build_buttons(&close_glyph);
        root.add_overlay(&actions);
        let overlay = Self {
            inner: Rc::new(Inner {
                root,
                picture,
                open_button,
                close_glyph,
                ink,
                provider: style::install_provider(),
                active_job: Cell::new(None),
                next_job: Cell::new(0),
                path: RefCell::new(None),
                view: RefCell::new(glib::WeakRef::new()),
                on_state: RefCell::new(None),
            }),
        };
        widgets::attach_interactions(&overlay, &close_button);
        overlay.apply_theme(theme);
        overlay
    }

    pub fn set_view(&self, view: &TermView) {
        self.inner.view.borrow().set(Some(view));
    }

    pub fn on_state(&self, cb: impl Fn(bool) + 'static) {
        self.inner.on_state.replace(Some(Box::new(cb)));
    }

    pub fn widget(&self) -> &gtk::Overlay {
        &self.inner.root
    }

    pub(super) fn open_widget(&self) -> &gtk::Button {
        &self.inner.open_button
    }

    pub(super) fn current_path(&self) -> Option<PathBuf> {
        self.inner.path.borrow().clone()
    }

    pub fn is_open(&self) -> bool {
        self.inner.root.is_visible()
    }

    pub fn open_from(
        &self,
        view: &TermView,
        source: ImageSource,
        path: Option<PathBuf>,
        label: &str,
    ) {
        self.set_view(view);
        let worker = view.worker();
        let job = self.inner.next_job.get().wrapping_add(1);
        self.inner.next_job.set(job);
        self.inner.active_job.set(Some(job));
        self.inner.open_button.set_label(label);
        self.inner.open_button.set_visible(path.is_some());
        *self.inner.path.borrow_mut() = path;
        self.inner.picture.set_paintable(gdk::Paintable::NONE);
        self.inner.root.set_visible(true);
        if let Some(v) = self.inner.view.borrow().upgrade() {
            v.set_picture_open(true);
        }
        if let Some(cb) = self.inner.on_state.borrow().as_ref() {
            cb(true);
        }
        worker.send(WorkerJob::Overlay(
            job,
            (MAX_OVERLAY_SIDE, MAX_OVERLAY_SIDE),
            source,
        ));
    }

    pub fn finish_decode(&self, job: u64, image: Option<ReadyPixels>) {
        if self.inner.active_job.get() != Some(job) {
            return;
        }
        match image.and_then(memory_texture) {
            Some(texture) => self.inner.picture.set_paintable(Some(&texture)),
            None => {
                log_error!("the image could not be decoded");
                self.close();
            }
        }
    }

    pub fn close(&self) {
        if !self.is_open() {
            return;
        }
        self.inner.active_job.set(None);
        *self.inner.path.borrow_mut() = None;
        self.inner.picture.set_paintable(gdk::Paintable::NONE);
        self.inner.open_button.set_visible(false);
        self.inner.root.set_visible(false);
        if let Some(view) = self.inner.view.borrow().upgrade() {
            view.set_picture_open(false);
            view.grab_focus();
        }
        if let Some(cb) = self.inner.on_state.borrow().as_ref() {
            cb(false);
        }
    }

    pub fn apply_theme(&self, theme: &Theme) {
        self.inner.ink.set(theme.text);
        self.inner
            .provider
            .load_from_string(&style::overlay_css(theme));
        self.inner.close_glyph.queue_draw();
    }
}
