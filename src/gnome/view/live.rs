use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::files::home_dir;
use trame::log_error;
use trame::preferences::DEFAULT_HISTORY_LINES;
use trame::session::pty::default_shell;
use trame::session::{Session, SessionEvent, SessionId, SessionOptions};

use super::TermView;
use super::metrics::{self, Metrics};
use crate::gnome::events;

const FIRST_SESSION: SessionId = SessionId(1);

pub fn fit(view: &TermView, width: i32, height: i32) {
    let Metrics {
        width: cell_width,
        height: cell_height,
        ..
    } = metrics::current(view);
    let cols = (width as f32 / cell_width).floor() as usize;
    let rows = (height as f32 / cell_height).floor() as usize;
    if cols == 0 || rows == 0 {
        return;
    }
    let imp = view.imp();
    let mut session = imp.session.borrow_mut();
    match session.as_mut() {
        Some(session) => session.resize(rows, cols),
        None => *session = start(view, rows, cols),
    }
    if let Some(session) = session.as_mut() {
        session.set_cell_pixels(cell_width.round() as u16, cell_height.round() as u16);
    }
}

fn start(view: &TermView, rows: usize, cols: usize) -> Option<Session> {
    let (notify, receiver) = events::channel();
    let shell = default_shell();
    let home = home_dir();
    let options = SessionOptions {
        shell: &shell,
        cwd: home.as_deref(),
        rows,
        cols,
        history: DEFAULT_HISTORY_LINES,
        colors: view.imp().theme.borrow().reported_colors(),
        transcript: &[],
    };
    let session = Session::spawn(FIRST_SESSION, &options, notify)
        .inspect_err(|error| log_error!("the shell could not start: {error}"))
        .ok()?;
    let watched = view.downgrade();
    events::listen(receiver, move |event| {
        watched.upgrade().is_some_and(|view| handle(&view, event))
    });
    Some(session)
}

fn handle(view: &TermView, event: SessionEvent) -> bool {
    match event {
        SessionEvent::Output(_) => {
            if let Some(session) = view.imp().session.borrow().as_ref() {
                session.take_output();
            }
            view.queue_draw();
            true
        }
        SessionEvent::Exited(_) => {
            if let Some(exit) = view.imp().exit.borrow().as_ref() {
                exit();
            }
            false
        }
    }
}

pub fn write(view: &TermView, bytes: Vec<u8>) {
    if let Some(session) = view.imp().session.borrow().as_ref() {
        session.terminal().screen_mut().reset_display();
        session.write(bytes);
        view.queue_draw();
    }
}
