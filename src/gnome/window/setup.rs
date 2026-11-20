use std::cell::RefCell;
use std::rc::{Rc, Weak};

use adw::prelude::*;
use trame::desk::Placement;
use trame::host::key::WindowKey;
use trame::host::update::UpdateStatus;
use trame::preferences::Recall;
use trame::session::Notifier;
use trame::theme::Appearance;
use trame::workspace::WindowSnapshot;

use super::{WindowCtx, layers, timer};
use crate::gnome::canvas::{LayerCanvas, ScenePainter};
use crate::gnome::events::unbounded_listen;
use crate::gnome::fleet::{Fleet, OpenSpec};
use crate::gnome::notify::GnomeNotices;
use crate::gnome::panes::PaneViews;
use crate::gnome::picture::PictureOverlay;
use crate::gnome::state::WindowState;

pub(super) struct CtxParts<'a> {
    pub app: &'a adw::Application,
    pub fleet: &'a Weak<RefCell<Fleet>>,
    pub key: WindowKey,
    pub window: adw::ApplicationWindow,
    pub state: Rc<RefCell<WindowState>>,
    pub session_slot: Rc<RefCell<Weak<WindowCtx>>>,
}

pub(super) fn init_desk(
    state: &Rc<RefCell<WindowState>>,
    spec: OpenSpec,
    update: &UpdateStatus,
    snap: Option<WindowSnapshot>,
) -> Result<(), String> {
    let mut s = state.borrow_mut();
    s.desk.saver.remember_window(snap);
    if matches!(update, UpdateStatus::Failed) {
        let eff = s.desk.update_failed();
        s.apply(eff);
    }
    open_initial_tabs(&mut s, spec)?;
    if !s.desk.preferences.recalls(Recall::Text) {
        s.desk.transcripts.clear();
    }
    Ok(())
}

fn open_initial_tabs(s: &mut WindowState, spec: OpenSpec) -> Result<(), String> {
    match spec {
        OpenSpec::Normal(saved, folder) => open_normal(s, saved.as_ref(), folder),
        OpenSpec::Torn(torn) => {
            let _ = s.desk.receive_tab(torn, None);
            s.changed();
            Ok(())
        }
        OpenSpec::Moved(moved) => {
            let eff = s.desk.receive_instance(moved, None);
            s.apply(eff);
            Ok(())
        }
    }
}

fn open_normal(
    s: &mut WindowState,
    saved: Option<&trame::workspace::Snapshot>,
    folder: Option<std::path::PathBuf>,
) -> Result<(), String> {
    let restored = s.desk.saver.owns() && s.restore_session(saved);
    if !restored || folder.is_some() {
        let eff = s.desk.open_at(Placement::Instance, folder);
        s.apply(eff);
        if s.desk.workspace.is_empty() {
            return Err("the first shell could not start".into());
        }
    }
    Ok(())
}

pub(super) fn assemble_ctx(parts: CtxParts<'_>) -> Rc<WindowCtx> {
    let panes = RefCell::new(PaneViews::new());
    let picture = PictureOverlay::shared(&parts.state.borrow().desk.theme);
    let chrome = LayerCanvas::new();
    let overlay = LayerCanvas::new();
    let stack = gtk::Overlay::new();
    stack.set_child(Some(&chrome));
    stack.add_overlay(panes.borrow().widget());
    stack.add_overlay(&overlay);
    stack.add_overlay(picture.widget());
    parts.window.set_content(Some(&stack));
    let ctx = Rc::new(WindowCtx {
        key: parts.key,
        app: parts.app.clone(),
        fleet: parts.fleet.clone(),
        window: parts.window,
        state: parts.state,
        panes,
        picture,
        notices: RefCell::new(GnomeNotices::default()),
        chrome,
        overlay,
        chrome_painter: RefCell::new(ScenePainter::default()),
        overlay_painter: RefCell::new(ScenePainter::default()),
        timer: RefCell::new(None),
    });
    *parts.session_slot.borrow_mut() = Rc::downgrade(&ctx);
    layers::wire_canvases(&ctx);
    ctx
}

pub(super) fn current_appearance() -> Appearance {
    let manager = adw::StyleManager::default();
    if manager.system_supports_color_schemes() && !manager.is_dark() {
        Appearance::Light
    } else {
        Appearance::Dark
    }
}

pub(super) fn session_notifier(slot: &Rc<RefCell<Weak<WindowCtx>>>) -> Notifier {
    let target = Rc::clone(slot);
    let sender = unbounded_listen(move |event| {
        if let Some(c) = target.borrow().upgrade() {
            timer::on_session_event(&c, event);
        }
        true
    });
    std::sync::Arc::new(move |event| {
        let _ = sender.send_blocking(event);
    })
}
