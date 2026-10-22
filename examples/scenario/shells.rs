use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use trame::session::pty::{ShellCommand, default_shell};
use trame::session::{Notifier, Session, SessionId, SessionOptions};
use trame::term::ReportedColors;
use trame::text::CellMetrics;
use trame::ui::FrameLayout;
use trame::workspace::{Edge, Workspace};

const COMMANDS: [&str; 3] = [
    "ls src\r",
    "git -c color.ui=always log --oneline --graph -12\r",
    "git status\r",
];

pub struct Scenario {
    pub workspace: Workspace,
    pub sessions: HashMap<SessionId, Session>,
    shell: ShellCommand,
    colors: ReportedColors,
    cwd: PathBuf,
}

impl Scenario {
    fn spawn(&mut self) -> Result<SessionId, Box<dyn Error>> {
        let id = SessionId(self.sessions.len() as u64 + 1);
        let options = SessionOptions {
            shell: &self.shell,
            cwd: Some(&self.cwd),
            rows: 24,
            cols: 80,
            colors: self.colors,
        };
        let notify: Notifier = Arc::new(|_| {});
        self.sessions
            .insert(id, Session::spawn(id, &options, notify)?);
        Ok(id)
    }

    pub fn label(&self, id: SessionId) -> String {
        self.sessions
            .get(&id)
            .map_or_else(String::new, Session::label)
    }

    pub fn resize(&mut self, frame: &FrameLayout, cell: &CellMetrics) {
        for pane in &frame.panes {
            let (rows, cols) = pane.grid_size(cell);
            for tab in &pane.tabs {
                if let Some(session) = self.sessions.get_mut(tab) {
                    session.resize(rows, cols);
                }
            }
        }
    }

    pub fn type_commands(&self, frame: &FrameLayout) {
        for (index, pane) in frame.panes.iter().enumerate() {
            let session = pane.active.and_then(|id| self.sessions.get(&id));
            if let Some(session) = session {
                session.write(COMMANDS[index % COMMANDS.len()]);
            }
        }
    }
}

pub fn build(colors: ReportedColors) -> Result<Scenario, Box<dyn Error>> {
    let mut scenario = Scenario {
        workspace: Workspace::new(),
        sessions: HashMap::new(),
        shell: default_shell(),
        colors,
        cwd: env::current_dir()?,
    };
    let first = scenario.spawn()?;
    scenario.workspace.add_instance(Some("trame".into()), first);
    let second = scenario.spawn()?;
    scenario.workspace.add_tab(second);
    let third = scenario.spawn()?;
    scenario.workspace.split_active(third, Edge::Right);
    let fourth = scenario.spawn()?;
    scenario
        .workspace
        .add_instance(Some("serveur de dev".into()), fourth);
    scenario.workspace.active = 0;
    Ok(scenario)
}
