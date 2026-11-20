mod actions;
mod input;
mod layers;
mod picker;
mod pointer;
mod setup;
mod timer;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use adw::prelude::*;
use trame::desk::DeskSetup;
use trame::host::key::WindowKey;
use trame::host::persist::{self, Slot};
use trame::host::update::UpdateStatus;
use trame::preferences::Preferences;
use trame::workspace::WindowSnapshot;

use super::canvas::{LayerCanvas, ScenePainter};
use super::fleet::{Fleet, OpenSpec, WindowHandle};
use super::notify::GnomeNotices;
use super::panes::PaneViews;
use super::picture::PictureOverlay;
use super::state::WindowState;

const APP_TITLE: &str = "trame";
const DEFAULT_WIDTH: i32 = 1180;
const DEFAULT_HEIGHT: i32 = 740;
const MIN_WIDTH: i32 = 520;
const MIN_HEIGHT: i32 = 320;

pub struct WindowCtx {
    pub key: WindowKey,
    pub app: adw::Application,
    pub fleet: Weak<RefCell<Fleet>>,
    pub window: adw::ApplicationWindow,
    pub state: Rc<RefCell<WindowState>>,
    pub panes: RefCell<PaneViews>,
    pub picture: PictureOverlay,
    pub notices: RefCell<GnomeNotices>,
    pub chrome: LayerCanvas,
    pub overlay: LayerCanvas,
    pub chrome_painter: RefCell<ScenePainter>,
    pub overlay_painter: RefCell<ScenePainter>,
    pub timer: RefCell<Option<gtk::glib::SourceId>>,
}

pub struct BuildRequest<'a> {
    pub app: &'a adw::Application,
    pub fleet: &'a Weak<RefCell<Fleet>>,
    pub key: WindowKey,
    pub slot: Slot,
    pub shared: Option<Preferences>,
    pub spec: OpenSpec,
    pub update: &'a UpdateStatus,
}

pub fn build_handle(req: BuildRequest<'_>) -> Result<WindowHandle, String> {
    let (prefs, snap) = initial_prefs(&req.spec, req.shared, &req.slot);
    let (window, w, h) = build_gtk_window(req.app, snap);
    let session_slot: Rc<RefCell<Weak<WindowCtx>>> = Rc::new(RefCell::new(Weak::new()));
    let state = Rc::new(RefCell::new(WindowState::new(
        DeskSetup {
            preferences: prefs,
            slot: req.slot,
            notifier: setup::session_notifier(&session_slot),
            appearance: setup::current_appearance(),
            icon_dir: persist::icon_dir(),
        },
        (w as f32, h as f32),
        window.scale_factor().max(1) as f32,
    )?));
    setup::init_desk(&state, req.spec, req.update, snap)?;
    let ctx = setup::assemble_ctx(setup::CtxParts {
        app: req.app,
        fleet: req.fleet,
        key: req.key,
        window,
        state: Rc::clone(&state),
        session_slot,
    });
    let c = Rc::clone(&ctx);
    let refresh: Box<dyn Fn()> = Box::new(move || timer::refresh(&c));
    layers::window_signals(&ctx);
    input::attach(&ctx);
    ctx.window.present();
    timer::refresh(&ctx);
    Ok(WindowHandle {
        key: req.key,
        window: ctx.window.clone(),
        state,
        refresh,
    })
}

fn initial_prefs(
    spec: &OpenSpec,
    shared: Option<Preferences>,
    slot: &Slot,
) -> (Preferences, Option<WindowSnapshot>) {
    let saved = match spec {
        OpenSpec::Normal(snap, _) => snap.as_ref(),
        _ => None,
    };
    let prefs = shared
        .or_else(|| saved.map(|s| s.preferences.clone()))
        .unwrap_or_default()
        .sanitized();
    let snap = saved.and_then(|s| s.window).filter(|_| slot.owns());
    (prefs, snap)
}

fn build_gtk_window(
    app: &adw::Application,
    snap: Option<WindowSnapshot>,
) -> (adw::ApplicationWindow, i32, i32) {
    let (w, h, max) = snap.map_or((DEFAULT_WIDTH, DEFAULT_HEIGHT, false), |s| {
        (
            s.width.max(MIN_WIDTH as u32) as i32,
            s.height.max(MIN_HEIGHT as u32) as i32,
            s.maximized,
        )
    });
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .default_width(w)
        .default_height(h)
        .decorated(false)
        .maximized(max)
        .build();
    window.set_size_request(MIN_WIDTH, MIN_HEIGHT);
    (window, w, h)
}
