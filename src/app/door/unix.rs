use std::fs;
use std::io::{self, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

const SOCKET_NAME: &str = "trame.sock";
const PATIENCE: Duration = Duration::from_secs(1);

pub struct Entrance {
    listener: UnixListener,
}

impl Entrance {
    pub fn open(folder: &Path) -> io::Result<Self> {
        let path = socket_path(folder);
        match fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
        Ok(Self {
            listener: UnixListener::bind(path)?,
        })
    }

    pub fn next_visitor(&mut self) -> io::Result<UnixStream> {
        let (visitor, _) = self.listener.accept()?;
        visitor.set_read_timeout(Some(PATIENCE))?;
        Ok(visitor)
    }
}

pub fn ring(folder: &Path, message: &[u8]) -> io::Result<()> {
    UnixStream::connect(socket_path(folder))?.write_all(message)
}

fn socket_path(folder: &Path) -> PathBuf {
    folder.join(SOCKET_NAME)
}
