mod role;
mod saver;
#[cfg(test)]
mod tests;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use trame::files::{home_dir, write_atomic};
use trame::preferences::{HostEntry, SshConfig};
use trame::workspace::Snapshot;

pub use role::Role;
pub use saver::Saver;

const APP_DIR: &str = "trame";
const FILE_NAME: &str = "session.json";
const TRANSCRIPT_DIR: &str = "text";
const ICON_DIR: &str = "icons";
const SSH_DIR: &str = ".ssh";
const SSH_CONFIG: &str = "config";

pub fn ssh_config() -> Option<SshConfig> {
    home_dir().map(|home| SshConfig::at(home.join(SSH_DIR).join(SSH_CONFIG)))
}

pub fn ssh_entry(name: &str) -> Option<HostEntry> {
    let config = ssh_config()?;
    config.entry(name).unwrap_or_else(|error| {
        eprintln!("trame: the SSH config could not be read: {error}");
        None
    })
}

#[cfg(windows)]
fn state_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn state_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join("Library").join("Application Support"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn state_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| home_dir().map(|home| home.join(".local").join("state")))
}

fn app_dir() -> Option<PathBuf> {
    state_dir().map(|dir| dir.join(APP_DIR))
}

pub fn claim() -> Role {
    Role::claim(app_dir())
}

fn session_path() -> Option<PathBuf> {
    app_dir().map(|dir| dir.join(FILE_NAME))
}

pub fn icon_dir() -> Option<PathBuf> {
    app_dir().map(|dir| dir.join(ICON_DIR))
}

pub fn load(role: &mut Role) -> Option<Snapshot> {
    load_from(&session_path()?, role)
}

fn load_from(path: &Path, role: &mut Role) -> Option<Snapshot> {
    let problem = match fs::read_to_string(path) {
        Ok(text) => match Snapshot::from_json(&text) {
            Ok(snapshot) => return Some(snapshot),
            Err(error) => error.to_string(),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
        Err(error) => format!("the session file could not be read: {error}"),
    };
    eprintln!("trame: {problem}, starting a fresh session");
    if role.owns() && !keep_aside(path) {
        *role = Role::Guest;
    }
    None
}

fn keep_aside(path: &Path) -> bool {
    let aside = path.with_extension("json.bad");
    let moved = fs::rename(path, &aside);
    if let Err(error) = &moved {
        eprintln!("trame: the unreadable session file stays as it is: {error}");
    }
    moved.is_ok()
}

pub fn write(path: &Path, snapshot: &Snapshot) -> io::Result<()> {
    let text = snapshot.to_json().map_err(io::Error::other)?;
    write_atomic(path, text.as_bytes())
}
