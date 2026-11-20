use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::time::Instant;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::display::TERMINAL_FONT_SIZE;
use trame::session::Session;
use trame::text::ImageSource;
use trame::theme::{Appearance, Theme};
use trame::ui::terminal::links::LinkCache;

use super::hover::HoverState;
use super::images::PreviewCache;
use super::live::BoundSession;
use super::metrics::Metrics;
use super::{images, input, live, paint, theme};
use crate::gnome::worker::{ImageWorker, ReadyPixels};

pub(super) type PictureOpenFn = Box<dyn Fn(ImageSource, Option<PathBuf>)>;
pub(super) type OverlayReadyFn = Box<dyn Fn(u64, Option<ReadyPixels>)>;
pub(super) type ThemeChangedFn = Box<dyn Fn(&Theme)>;
pub(super) type PrimarySelectFn = Box<dyn Fn(String)>;
pub(super) type ClickState = Option<(Instant, (usize, usize), usize)>;

#[derive(Clone, Copy, Default)]
pub(super) struct PointerState {
    pub position: Option<(f32, f32)>,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub cursor: &'static str,
    pub dragging: bool,
    pub reported_button: Option<trame::input::MouseButton>,
    pub last_cell: Option<(usize, usize)>,
}

pub struct TermView {
    pub(super) session: RefCell<Option<Session>>,
    pub(super) bound: RefCell<Option<BoundSession>>,
    pub(super) origin: RefCell<Option<PathBuf>>,
    pub(super) font_size: Cell<f32>,
    pub(super) detect_links: Cell<bool>,
    pub(super) scroll: Cell<f32>,
    pub(super) metrics: RefCell<Option<Metrics>>,
    pub(super) theme: RefCell<Theme>,
    pub(super) links: RefCell<LinkCache>,
    pub(super) images: RefCell<PreviewCache>,
    pub(super) hover: RefCell<HoverState>,
    pub(super) worker: RefCell<Option<ImageWorker>>,
    pub(super) pointer: Cell<PointerState>,
    pub(super) clicks: Cell<ClickState>,
    pub(super) picture_open: Cell<bool>,
    pub(super) picture: RefCell<Option<PictureOpenFn>>,
    pub(super) overlay_ready: RefCell<Option<OverlayReadyFn>>,
    pub(super) theme_changed: RefCell<Option<ThemeChangedFn>>,
    pub(super) primary_selection: RefCell<Option<PrimarySelectFn>>,
    pub(super) middle_click: RefCell<Option<Box<dyn Fn()>>>,
    pub(super) exit: RefCell<Option<Box<dyn Fn()>>>,
}

impl Default for TermView {
    fn default() -> Self {
        Self {
            session: RefCell::default(),
            bound: RefCell::default(),
            origin: RefCell::default(),
            font_size: Cell::new(TERMINAL_FONT_SIZE),
            detect_links: Cell::new(true),
            scroll: Cell::new(0.0),
            metrics: RefCell::default(),
            theme: RefCell::new(Theme::new(Appearance::Dark)),
            links: RefCell::default(),
            images: RefCell::default(),
            hover: RefCell::default(),
            worker: RefCell::default(),
            pointer: Cell::default(),
            clicks: Cell::default(),
            picture_open: Cell::new(false),
            picture: RefCell::default(),
            overlay_ready: RefCell::default(),
            theme_changed: RefCell::default(),
            primary_selection: RefCell::default(),
            middle_click: RefCell::default(),
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
        let _ = images::ensure_worker(&view);
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
