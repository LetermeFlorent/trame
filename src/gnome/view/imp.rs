use std::cell::RefCell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::session::Session;
use trame::theme::{Appearance, Theme};

use super::metrics::Metrics;
use super::{input, live, paint, theme};

pub struct TermView {
    pub(super) session: RefCell<Option<Session>>,
    pub(super) metrics: RefCell<Option<Metrics>>,
    pub(super) theme: RefCell<Theme>,
    pub(super) exit: RefCell<Option<Box<dyn Fn()>>>,
}

impl Default for TermView {
    fn default() -> Self {
        Self {
            session: RefCell::default(),
            metrics: RefCell::default(),
            theme: RefCell::new(Theme::new(Appearance::Dark)),
            exit: RefCell::default(),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for TermView {
    const NAME: &'static str = "TrameTermView";
    type Type = super::TermView;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for TermView {
    fn constructed(&self) {
        self.parent_constructed();
        let view = self.obj();
        view.set_focusable(true);
        view.set_hexpand(true);
        view.set_vexpand(true);
        view.set_overflow(gtk::Overflow::Hidden);
        theme::follow(&view);
        input::attach(&view);
    }
}

impl WidgetImpl for TermView {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        paint::draw(&self.obj(), snapshot);
    }

    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);
        live::fit(&self.obj(), width, height);
    }
}
