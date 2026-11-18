mod cursor;
mod images;
mod imp;
mod input;
mod keys;
mod live;
mod metrics;
mod paint;
mod text;
mod theme;

use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclassIsExt;

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
}

impl Default for TermView {
    fn default() -> Self {
        Self::new()
    }
}
