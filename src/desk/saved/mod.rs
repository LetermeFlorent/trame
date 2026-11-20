mod restore;
mod text;

use std::time::Instant;

use crate::preferences::Recall;
use crate::session::SessionId;
use crate::workspace::{TabSnapshot, WindowSnapshot};

use super::{Desk, Effect, Effects, Outcome};

impl Desk {
    pub fn save_deadline(
        &mut self,
        now: Instant,
        window: impl FnOnce() -> Option<WindowSnapshot>,
    ) -> Outcome<Option<Instant>> {
        self.saver.watch(&self.workspace);
        let due = self.saver.due();
        if due.is_none_or(|due| due > now) {
            return Outcome {
                value: due,
                effects: Effects::default(),
            };
        }
        Outcome {
            value: None,
            effects: self.save_session(false, window()),
        }
    }

    pub fn save_session(&mut self, resume: bool, window: Option<WindowSnapshot>) -> Effects {
        self.transcripts.assign(&self.workspace.tabs());
        let sessions = &self.sessions;
        let transcripts = &self.transcripts;
        let workspace = &self.workspace;
        let keeps_text = resume || self.preferences.recalls(Recall::Text);
        let keeps_remote = self.preferences.keep_remote_tasks;
        let describe = |id: SessionId| TabSnapshot {
            cwd: sessions.cwd(id),
            transcript: transcripts
                .key(id)
                .filter(|_| keeps_text || (keeps_remote && workspace.tab_host(id).is_some()))
                .map(str::to_owned),
            ..TabSnapshot::default()
        };
        let mut snapshot = self.workspace.snapshot(self.preferences.clone(), &describe);
        snapshot.resume = resume;
        snapshot.window = self.saver.remember_window(window);
        let kept = self.saver.keeps_session();
        self.saver.save(&snapshot);
        if self.saver.keeps_session() != kept {
            return Effect::Changed.into();
        }
        Effects::default()
    }
}
