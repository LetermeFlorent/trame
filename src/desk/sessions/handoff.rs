use std::path::PathBuf;

use crate::session::{Session, SessionId};

use super::{Sessions, Waiting};

pub struct Leaving {
    session: Option<Session>,
    origin: Option<PathBuf>,
    waiting: Option<Waiting>,
}

impl Sessions {
    pub fn hand_off(&mut self, id: SessionId) -> Leaving {
        self.answers.remove(&id);
        self.remotes.remove(&id);
        Leaving {
            session: self.map.remove(&id),
            origin: self.origins.remove(&id),
            waiting: self.waiting.remove(&id),
        }
    }

    pub fn take_in(&mut self, leaving: Leaving) -> SessionId {
        let id = self.reserve();
        if let Some(mut session) = leaving.session {
            session.hand_over(id, self.notify.clone());
            self.prepare(&session);
            self.map.insert(id, session);
        }
        if let Some(origin) = leaving.origin {
            self.origins.insert(id, origin);
        }
        if let Some(waiting) = leaving.waiting {
            self.waiting.insert(id, waiting);
        }
        id
    }
}
