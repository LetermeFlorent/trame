use std::time::Instant;

use trame::session::{SessionId, transcript_of};

use crate::app::running::Running;

impl Running {
    pub(super) fn transcript_deadline(&mut self) -> Option<Instant> {
        if !self.preferences.restore_text {
            return None;
        }
        if self.transcripts.due() <= Instant::now() {
            self.save_transcripts();
        }
        Some(self.transcripts.due())
    }

    pub fn save_on_exit(&mut self) {
        self.save_transcripts();
        self.save_session();
    }

    fn save_transcripts(&mut self) {
        self.transcripts.schedule();
        if !self.preferences.restore_text {
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
