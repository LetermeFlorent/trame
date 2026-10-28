use std::time::Instant;

use trame::preferences::Recall;
use trame::session::Session;

use crate::app::running::Running;

impl Running {
    pub(super) fn transcript_deadline(&mut self) -> Option<Instant> {
        if !self.saver.owns() || !self.preferences.recalls(Recall::Text) {
            return None;
        }
        if self.transcripts.due() <= Instant::now() {
            self.save_transcripts(false);
        }
        Some(self.transcripts.due())
    }

    pub fn save_on_exit(&mut self) {
        if !self.update.restarting {
            self.save_everything(false);
        }
    }

    pub(super) fn save_for_restart(&mut self) {
        self.save_everything(true);
    }

    fn save_everything(&mut self, resume: bool) {
        self.transcripts.flush();
        self.save_transcripts(resume);
        self.transcripts.flush();
        self.save_session(resume);
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
