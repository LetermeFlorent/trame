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
use trame::locale::system_language;
use trame::log_error;
use trame::text::ImageSource;
use trame::theme::{Rgba, Theme};

use super::view::TermView;
use super::worker::{ImageWorker, ReadyPixels, WorkerJob, memory_texture};
use rules::{KeyOutcome, MAX_OVERLAY_SIDE, overlay_key};

#[derive(Clone)]
pub struct PictureOverlay {
    inner: Rc<Inner>,
}

struct Inner {
    root: gtk::Overlay,
    picture: gtk::Picture,
    open_button: gtk::Button,
    close_glyph: gtk::DrawingArea,
    ink: Rc<Cell<Rgba>>,
    provider: gtk::CssProvider,
    active_job: Cell<Option<u64>>,
    next_job: Cell<u64>,
    path: RefCell<Option<PathBuf>>,
    view: glib::WeakRef<TermView>,
}

impl PictureOverlay {
    pub fn new(view: &TermView) -> Self {
        let theme = view.current_theme();
        let ink = Rc::new(Cell::new(theme.text));
        let close_glyph = style::close_icon(Rc::clone(&ink));
        let (root, picture) = widgets::build_scrim();
        let (actions, open_button, close_button) = widgets::build_buttons(&close_glyph);
        root.add_overlay(&actions);
        let watched = glib::WeakRef::new();
        watched.set(Some(view));
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
                view: watched,
            }),
        };
        widgets::attach_interactions(&overlay, &close_button);
        overlay.apply_theme(&theme);
        overlay
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

    pub fn open(&self, source: ImageSource, path: Option<PathBuf>, worker: &ImageWorker) {
        let job = self.inner.next_job.get().wrapping_add(1);
        self.inner.next_job.set(job);
        self.inner.active_job.set(Some(job));
        let label = system_language().strings().open_in_viewer;
        self.inner.open_button.set_label(label);
        self.inner.open_button.set_visible(path.is_some());
        *self.inner.path.borrow_mut() = path;
        self.inner.picture.set_paintable(gdk::Paintable::NONE);
        self.inner.root.set_visible(true);
        if let Some(view) = self.inner.view.upgrade() {
            view.set_picture_open(true);
        }
        let limit = (MAX_OVERLAY_SIDE, MAX_OVERLAY_SIDE);
        worker.send(WorkerJob::Overlay(job, limit, source));
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
        if let Some(view) = self.inner.view.upgrade() {
            view.set_picture_open(false);
            view.grab_focus();
        }
    }

    pub fn handle_key(&self, keyval: gdk::Key) -> bool {
        match overlay_key(self.is_open(), keyval == gdk::Key::Escape) {
            KeyOutcome::Pass => false,
            KeyOutcome::Consume => true,
            KeyOutcome::Close => {
                self.close();
                true
            }
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
