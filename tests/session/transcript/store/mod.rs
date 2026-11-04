mod reading;
mod writing;

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use trame::session::{SessionId, Transcripts};
use trame::term::Terminal;

use crate::sandbox::Sandbox;

const FIRST: SessionId = SessionId(1);
const SECOND: SessionId = SessionId(2);

impl Sandbox {
    fn text(&self) -> PathBuf {
        self.root.join("text")
    }

    fn transcripts(&self) -> Transcripts {
        Transcripts::new(Some(self.text()))
    }

    fn files(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.text())
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }
}

fn key_of(transcripts: &Transcripts, id: SessionId) -> String {
    transcripts
        .key(id)
        .expect("tab should have a key")
        .to_owned()
}

fn tab_with(text: &[u8]) -> Arc<Mutex<Terminal>> {
    let mut terminal = Terminal::new(4, 20, 100);
    terminal.feed(text);
    Arc::new(Mutex::new(terminal))
}

fn save_now(transcripts: &mut Transcripts, tabs: &[(SessionId, &Arc<Mutex<Terminal>>)]) {
    let ids: Vec<SessionId> = tabs.iter().map(|(id, _)| *id).collect();
    transcripts.save(&ids, |id| {
        tabs.iter()
            .find(|(tab, _)| *tab == id)
            .map(|(_, terminal)| Arc::clone(terminal))
    });
    transcripts.flush();
}
