use std::path::PathBuf;

use trame::host::update::UpdateStatus;
use trame::session::SessionEvent;
use trame::workspace::InstanceId;

use super::running::FolderTarget;

#[derive(Clone, Debug)]
pub enum UserEvent<K> {
    Session(K, SessionEvent),
    Update(UpdateStatus),
    Folder(K, FolderTarget, PathBuf),
    Icon(K, InstanceId, u64),
    Open(Option<PathBuf>),
}
