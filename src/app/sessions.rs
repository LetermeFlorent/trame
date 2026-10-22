use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use winit::event_loop::EventLoopProxy;

use trame::session::pty::{ShellCommand, default_shell};
use trame::session::{Notifier, Session, SessionId, SessionOptions};
use trame::term::ReportedColors;

use super::UserEvent;

pub struct ShellStart<'a> {
    pub cwd: Option<&'a Path>,
    pub size: (usize, usize),
    pub colors: ReportedColors,
    pub transcript: &'a [u8],
}

pub struct Sessions {
    map: HashMap<SessionId, Session>,
    origins: HashMap<SessionId, PathBuf>,
    next: u64,
    shell: ShellCommand,
    notify: Notifier,
}

impl Sessions {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        let proxy = Mutex::new(proxy);
        let notify: Notifier = Arc::new(move |event| {
            let proxy = proxy.lock().unwrap_or_else(PoisonError::into_inner);
            let _ = proxy.send_event(UserEvent::Session(event));
        });
        Self {
            map: HashMap::new(),
            origins: HashMap::new(),
            next: 0,
            shell: default_shell(),
            notify,
        }
    }

    pub fn reserve(&mut self) -> SessionId {
        self.next += 1;
        SessionId(self.next)
    }

    pub fn spawn(&mut self, id: SessionId, start: &ShellStart) -> io::Result<()> {
        let options = SessionOptions {
            shell: &self.shell,
            cwd: start.cwd,
            rows: start.size.0,
            cols: start.size.1,
            colors: start.colors,
            transcript: start.transcript,
        };
        let session = Session::spawn(id, &options, self.notify.clone())?;
        self.map.insert(id, session);
        if let Some(path) = start.cwd {
            self.origins.insert(id, path.to_path_buf());
        }
        Ok(())
    }

    pub fn get(&self, id: SessionId) -> Option<&Session> {
        self.map.get(&id)
    }

    pub fn resize(&mut self, id: SessionId, size: (usize, usize)) {
        if let Some(session) = self.map.get_mut(&id) {
            session.resize(size.0, size.1);
        }
    }

    pub fn remove(&mut self, id: SessionId) -> Option<Session> {
        self.origins.remove(&id);
        self.map.remove(&id)
    }

    pub fn cwd(&self, id: SessionId) -> Option<PathBuf> {
        let known = self.map.get(&id).and_then(Session::cwd);
        known.or_else(|| self.origins.get(&id).cloned())
    }

    pub fn map(&self) -> &HashMap<SessionId, Session> {
        &self.map
    }

    pub fn label(&self, id: SessionId) -> String {
        self.map.get(&id).map_or_else(String::new, Session::label)
    }

    pub fn set_colors(&self, colors: ReportedColors) {
        for session in self.map.values() {
            session.terminal().screen_mut().set_reported_colors(colors);
        }
    }

    pub fn clear_output(&self) {
        for session in self.map.values() {
            session.take_output();
        }
    }
}
