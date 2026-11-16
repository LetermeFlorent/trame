use std::io;
use std::path::PathBuf;

use crate::desk::{Desk, Effect, Effects, Grids, ShellStart};
use crate::host::quit::Quit;
use crate::log_error;
use crate::session::SessionId;

pub struct Starting {
    id: SessionId,
    cwd: Option<PathBuf>,
}

struct Revival<'a> {
    cwd: Option<PathBuf>,
    transcript: Option<&'a [u8]>,
    size: (usize, usize),
}

impl Desk {
    pub(super) fn queue_starts(&mut self, starts: Vec<(SessionId, Option<PathBuf>)>) -> Effects {
        let queued = starts.into_iter().map(|(id, cwd)| Starting { id, cwd });
        self.starting.extend(queued);
        Effects::from([Effect::Relayout, Effect::StartQueued, Effect::Changed])
    }

    pub fn start_queued(&mut self, grids: &Grids) {
        for Starting { id, cwd } in std::mem::take(&mut self.starting) {
            let revival = Revival {
                cwd,
                transcript: None,
                size: grids.size_of(id),
            };
            if let Err(error) = self.start_shell(id, &revival) {
                log_error!("the shell could not start: {error}");
            }
        }
        if self.workspace.is_empty() {
            self.quit_with(Quit::Close);
        }
    }

    pub fn start_shown(&mut self, grids: &Grids) -> bool {
        let shown: Vec<SessionId> = grids
            .shown()
            .iter()
            .copied()
            .filter(|id| self.sessions.is_waiting(*id))
            .collect();
        for id in &shown {
            self.start_waiting(*id, grids.size_of(*id));
        }
        !shown.is_empty()
    }

    fn start_waiting(&mut self, id: SessionId, size: (usize, usize)) {
        let Some(waiting) = self.sessions.take_waiting(id) else {
            return;
        };
        let text = waiting
            .transcript
            .map(|key| self.transcripts.read(&key).unwrap_or_default());
        let revival = Revival {
            cwd: self.sessions.cwd(id),
            transcript: text.as_deref(),
            size,
        };
        if let Err(error) = self.start_shell(id, &revival) {
            log_error!("a saved shell could not start: {error}");
        }
    }

    fn start_shell(&mut self, id: SessionId, revival: &Revival) -> io::Result<()> {
        let start = ShellStart {
            cwd: revival.cwd.as_deref(),
            size: revival.size,
            colors: self.theme.reported_colors(),
            transcript: revival.transcript.unwrap_or_default(),
        };
        let spawned = self.sessions.spawn(id, &start);
        match spawned {
            Ok(()) => self.greet(id, revival.transcript.is_none()),
            Err(_) => {
                self.workspace.close_tab(id);
            }
        }
        spawned
    }
}
