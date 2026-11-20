use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::files::home_dir;
use trame::log_error;
use trame::preferences::DEFAULT_HISTORY_LINES;
use trame::session::pty::default_shell;
use trame::session::{Session, SessionEvent, SessionId, SessionOptions};

use super::{TermView, input};
use crate::gnome::events;

const FIRST_SESSION: SessionId = SessionId(1);

pub(super) fn start(view: &TermView, rows: usize, cols: usize) -> Option<Session> {
    let (notify, receiver) = events::channel();
    let shell = default_shell();
    let home = home_dir();
    *view.imp().origin.borrow_mut() = home.clone();
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
            input::refresh_hover(view);
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
