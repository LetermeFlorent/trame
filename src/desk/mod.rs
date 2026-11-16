mod effect;
mod grids;
pub mod hosts;
mod quit;
mod restarts;
pub mod sessions;
mod settings;
mod setup;
mod tabs;
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
pub use grids::{FALLBACK_GRID, Grids};
pub use hosts::{HostForm, written_by_hand};
pub use restarts::Restarts;
pub use sessions::{Leaving, Sessions, ShellStart, Waiting};
pub use setup::DeskSetup;
use tabs::Starting;
pub use tabs::{FolderTarget, Placement};
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
    starting: Vec<Starting>,
}
