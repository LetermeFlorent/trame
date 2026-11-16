mod effect;
mod quit;
mod restarts;
pub mod sessions;
mod setup;
mod update;

use crate::files::IconStore;
use crate::host::persist::Saver;
use crate::host::power::Power;
use crate::host::quit::Quit;
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::session::Transcripts;
use crate::theme::Theme;
use crate::workspace::Workspace;

pub use effect::{Effect, Effects, Outcome};
pub use restarts::Restarts;
pub use sessions::{Leaving, Sessions, ShellStart, Waiting};
pub use setup::DeskSetup;
pub use update::UpdateState;

pub struct Desk {
    pub workspace: Workspace,
    pub sessions: Sessions,
    pub preferences: Preferences,
    pub saver: Saver,
    pub transcripts: Transcripts,
    pub icons: IconStore,
    pub power: Power,
    pub theme: Theme,
    pub strings: &'static Strings,
    pub update: UpdateState,
    pub restarts: Restarts,
    quit: Option<Quit>,
}
