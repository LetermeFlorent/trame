use std::time::Instant;

use trame::preferences::Recall;
use trame::session::Session;

use crate::app::running::Running;

impl Running {
    pub(super) fn transcript_deadline(&mut self) -> Option<Instant> {
        if !self.desk.saver.owns() || !self.desk.preferences.recalls(Recall::Text) {
            return None;
        }
        if self.desk.transcripts.due() <= Instant::now() {
            self.save_transcripts(false);
        }
        Some(self.desk.transcripts.due())
    }

    pub fn save_on_exit(&mut self) {
        if !self.desk.update.restarting {
            self.save_everything(false);
        }
    }

    pub fn save_for_restart(&mut self) {
        self.save_everything(true);
    }

    fn save_everything(&mut self, resume: bool) {
        self.desk.transcripts.flush();
        self.save_transcripts(resume);
        self.desk.transcripts.flush();
        self.save_session(resume);
    }

    fn save_transcripts(&mut self, every_tab: bool) {
        self.desk.transcripts.schedule();
        if !self.desk.saver.owns() || !every_tab && !self.desk.preferences.recalls(Recall::Text) {
            return;
        }
        let tabs = self.desk.workspace.tabs();
        self.desk.transcripts.save(&tabs, |id| {
            self.desk.sessions.get(id).map(Session::shared_terminal)
        });
    }
}
