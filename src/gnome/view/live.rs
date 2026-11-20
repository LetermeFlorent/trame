use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::session::{Session, SessionId};
use trame::term::{Modes, Terminal};
use trame::ui::terminal::links::LinkCache;

use super::metrics::{self, Metrics};
use super::{TermView, spawn};

pub struct BoundSession {
    pub id: SessionId,
    pub terminal: Arc<Mutex<Terminal>>,
    pub cwd: Option<PathBuf>,
    pub writer: Box<dyn Fn(Vec<u8>)>,
    pub focus: Box<dyn Fn()>,
}

pub fn bind(view: &TermView, bound: BoundSession) {
    let imp = view.imp();
    *imp.session.borrow_mut() = None;
    *imp.origin.borrow_mut() = bound.cwd.clone();
    *imp.bound.borrow_mut() = Some(bound);
    *imp.links.borrow_mut() = LinkCache::default();
    view.queue_draw();
}

pub fn with_terminal<R>(
    view: &TermView,
    f: impl FnOnce(SessionId, &mut Terminal, Option<&Path>) -> R,
) -> Option<R> {
    let imp = view.imp();
    let cwd = session_cwd(view);
    if let Some(bound) = imp.bound.borrow().as_ref() {
        let mut terminal = bound.terminal.lock().unwrap_or_else(|e| e.into_inner());
        return Some(f(bound.id, &mut terminal, cwd.as_deref()));
    }
    let guard = imp.session.borrow();
    let session = guard.as_ref()?;
    let mut terminal = session.terminal();
    Some(f(session.id(), &mut terminal, cwd.as_deref()))
}

pub fn fit(view: &TermView, width: i32, height: i32) {
    if view.imp().bound.borrow().is_some() {
        return;
    }
    let Metrics {
        width: cw,
        height: ch,
        ..
    } = metrics::current(view);
    let (cols, rows) = (
        (width as f32 / cw).floor() as usize,
        (height as f32 / ch).floor() as usize,
    );
    if cols == 0 || rows == 0 {
        return;
    }
    let imp = view.imp();
    let mut session = imp.session.borrow_mut();
    match session.as_mut() {
        Some(session) => session.resize(rows, cols),
        None => *session = spawn::start(view, rows, cols),
    }
}

pub fn session_cwd(view: &TermView) -> Option<PathBuf> {
    let imp = view.imp();
    if let Some(bound) = imp.bound.borrow().as_ref() {
        return bound.cwd.clone().or_else(|| imp.origin.borrow().clone());
    }
    let known = imp.session.borrow().as_ref().and_then(Session::cwd);
    known.or_else(|| imp.origin.borrow().clone())
}

pub fn notify_focus(view: &TermView, entered: bool) {
    let imp = view.imp();
    if entered && let Some(bound) = imp.bound.borrow().as_ref() {
        (bound.focus)();
    }
    if with_terminal(view, |_, t, _| {
        t.screen().modes().contains(Modes::FOCUS_EVENTS)
    }) == Some(true)
    {
        write(
            view,
            if entered {
                b"\x1b[I".to_vec()
            } else {
                b"\x1b[O".to_vec()
            },
        );
    }
}

pub fn write(view: &TermView, bytes: Vec<u8>) {
    if bytes.is_empty() {
        return;
    }
    let imp = view.imp();
    if let Some(bound) = imp.bound.borrow().as_ref() {
        (bound.writer)(bytes);
        view.queue_draw();
        return;
    }
    if let Some(session) = imp.session.borrow().as_ref() {
        session.terminal().screen_mut().reset_display();
        session.write(bytes);
        view.queue_draw();
    }
}
