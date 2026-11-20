mod hover;
mod images;
mod imp;
mod input;
pub mod live;
mod metrics;
pub mod paint;
mod spawn;
mod theme;

use std::path::PathBuf;

use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use trame::text::ImageSource;
use trame::theme::Theme;
use trame::ui::terminal::links::LinkCache;

use super::worker::{ImageWorker, ReadyPixels};
pub use live::BoundSession;

glib::wrapper! {
    pub struct TermView(ObjectSubclass<imp::TermView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl TermView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn bind(&self, session: BoundSession) {
        live::bind(self, session);
    }

    pub fn set_theme(&self, theme: Theme) {
        theme::apply_theme(self, theme);
    }

    pub fn set_font_size(&self, size: f32) {
        metrics::set_font_size(self, size);
    }

    pub fn set_detect_links(&self, detect: bool) {
        self.imp().detect_links.set(detect);
    }

    pub fn set_cwd(&self, cwd: Option<PathBuf>) {
        let mut slot = self.imp().origin.borrow_mut();
        if *slot != cwd {
            *slot = cwd;
            *self.imp().links.borrow_mut() = LinkCache::default();
        }
    }

    pub fn set_scroll(&self, scroll: f32) {
        self.imp().scroll.set(scroll);
    }

    pub fn cell_size(&self) -> (f32, f32) {
        let m = metrics::current(self);
        (m.width, m.height)
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

    pub fn on_primary_selection(&self, selected: impl Fn(String) + 'static) {
        self.imp()
            .primary_selection
            .replace(Some(Box::new(selected)));
    }

    pub fn on_middle_click(&self, paste: impl Fn() + 'static) {
        self.imp().middle_click.replace(Some(Box::new(paste)));
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
