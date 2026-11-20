use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use gtk::glib;
use trame::theme::Rgba;

use crate::gnome::view::TermView;

pub(super) type StateFn = Box<dyn Fn(bool)>;

pub(super) struct Inner {
    pub root: gtk::Overlay,
    pub picture: gtk::Picture,
    pub open_button: gtk::Button,
    pub close_glyph: gtk::DrawingArea,
    pub ink: Rc<Cell<Rgba>>,
    pub provider: gtk::CssProvider,
    pub active_job: Cell<Option<u64>>,
    pub next_job: Cell<u64>,
    pub path: RefCell<Option<PathBuf>>,
    pub view: RefCell<glib::WeakRef<TermView>>,
    pub on_state: RefCell<Option<StateFn>>,
}
