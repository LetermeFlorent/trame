mod closing;
mod commands;
mod deadline;
mod edit;
mod effect;
mod grids;
pub mod hosts;
mod output;
mod paste;
mod quit;
mod restarts;
mod saved;
pub mod sessions;
mod settings;
mod setup;
mod system;
mod tabs;
mod transfer;
mod update;
mod write;

use crate::files::IconStore;
use crate::host::persist::Saver;
use crate::host::power::Power;
use crate::host::quit::Quit;
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::session::Transcripts;
use crate::theme::Theme;
use crate::workspace::Workspace;

pub use closing::{Closing, Confirm, Question};
pub use edit::Edit;
pub use effect::{Effect, Effects, Outcome};
pub use grids::{FALLBACK_GRID, Grids};
pub use hosts::{HostForm, written_by_hand};
pub use output::Seen;
pub use paste::PendingPaste;
pub use restarts::Restarts;
pub use sessions::{Leaving, Sessions, ShellStart, Waiting};
pub use setup::DeskSetup;
use tabs::Starting;
pub use tabs::{FolderTarget, Placement};
pub use transfer::{MovedInstance, TornTab};
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
    pub closing: Option<Closing>,
    quit: Option<Quit>,
    starting: Vec<Starting>,
}
