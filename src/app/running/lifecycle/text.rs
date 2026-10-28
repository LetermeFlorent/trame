use std::time::Instant;

use trame::preferences::Recall;
use trame::session::{SessionId, transcript_of};

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
        if !self.restarting {
            self.save_everything(false);
        }
    }

    pub(super) fn save_for_restart(&mut self) {
        self.save_everything(true);
    }

    fn save_everything(&mut self, resume: bool) {
        self.save_transcripts(resume);
        self.save_session(resume);
    }

    fn save_transcripts(&mut self, every_tab: bool) {
        self.transcripts.schedule();
        if !self.saver.owns() || !every_tab && !self.preferences.recalls(Recall::Text) {
            return;
        }
        let tabs = self.workspace.tabs();
        self.transcripts.assign(&tabs);
        for id in &tabs {
            self.save_transcript(*id);
        }
        self.transcripts.prune(&tabs);
    }

    fn save_transcript(&mut self, id: SessionId) {
        let Some(session) = self.sessions.get(id) else {
            return;
        };
        let (generation, bytes) = {
            let terminal = session.terminal();
            let generation = terminal.generation();
            if self.transcripts.is_current(id, generation) {
                return;
            }
            (generation, transcript_of(&terminal))
        };
        if let Err(error) = self.transcripts.write(id, generation, &bytes) {
            eprintln!("trame: a tab text could not be saved: {error}");
        }
    }
}
