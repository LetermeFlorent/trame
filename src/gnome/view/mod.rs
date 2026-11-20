mod hover;
mod images;
mod imp;
mod input;
mod live;
mod metrics;
pub mod paint;
mod theme;

use std::path::PathBuf;

use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use trame::text::ImageSource;
use trame::theme::Theme;

use super::worker::{ImageWorker, ReadyPixels};

glib::wrapper! {
    pub struct TermView(ObjectSubclass<imp::TermView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl TermView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn on_exit(&self, exit: impl Fn() + 'static) {
        self.imp().exit.replace(Some(Box::new(exit)));
    }

    pub fn on_picture(&self, open: impl Fn(ImageSource, Option<PathBuf>) + 'static) {
        self.imp().picture.replace(Some(Box::new(open)));
    }

    pub fn on_overlay_ready(&self, ready: impl Fn(u64, Option<ReadyPixels>) + 'static) {
        self.imp().overlay_ready.replace(Some(Box::new(ready)));
    }

    pub fn on_theme(&self, changed: impl Fn(&Theme) + 'static) {
        self.imp().theme_changed.replace(Some(Box::new(changed)));
    }

    pub fn current_theme(&self) -> Theme {
        self.imp().theme.borrow().clone()
    }

    pub fn worker(&self) -> ImageWorker {
        images::ensure_worker(self)
    }

    pub fn set_picture_open(&self, open: bool) {
        self.imp().picture_open.set(open);
        if open {
            hover::clear_hover(self);
        } else {
            input::refresh_hover(self);
        }
    }

    pub fn picture_open(&self) -> bool {
        self.imp().picture_open.get()
    }
}

impl Default for TermView {
    fn default() -> Self {
        Self::new()
    }
}
