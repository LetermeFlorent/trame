use std::time::Instant;

use crate::preferences::Recall;
use crate::session::Session;
use crate::workspace::WindowSnapshot;

use crate::desk::{Desk, Effects};

impl Desk {
    pub fn transcript_deadline(&mut self, now: Instant) -> Option<Instant> {
        if !self.saver.owns() || !self.preferences.recalls(Recall::Text) {
            return None;
        }
        if self.transcripts.due() <= now {
            self.save_transcripts(false);
        }
        Some(self.transcripts.due())
    }

    pub fn save_on_exit(&mut self, window: Option<WindowSnapshot>) -> Effects {
        if self.update.restarting {
            return Effects::default();
        }
        self.save_everything(false, window)
    }

    pub fn save_for_restart(&mut self, window: Option<WindowSnapshot>) -> Effects {
        self.save_everything(true, window)
    }

    fn save_everything(&mut self, resume: bool, window: Option<WindowSnapshot>) -> Effects {
        self.transcripts.flush();
        self.save_transcripts(resume);
        self.transcripts.flush();
        self.save_session(resume, window)
    }

    fn save_transcripts(&mut self, every_tab: bool) {
        self.transcripts.schedule();
        if !self.saver.owns() || !every_tab && !self.preferences.recalls(Recall::Text) {
            return;
        }
        let tabs = self.workspace.tabs();
        self.transcripts.save(&tabs, |id| {
            self.sessions.get(id).map(Session::shared_terminal)
        });
    }
}
