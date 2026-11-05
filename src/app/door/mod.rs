#[cfg(test)]
mod tests;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use trame::log_error;

#[cfg(unix)]
use unix::{Entrance, ring};
#[cfg(windows)]
use windows::{Entrance, ring};

const OPEN: &[u8] = b"open\n";
const LONGEST: u64 = 32 * 1024;
const THREAD_NAME: &str = "trame-door";
const ATTEMPTS: u32 = 20;
const KNOCKS: u32 = 4;
const PAUSE: Duration = Duration::from_millis(250);

pub fn knock(folder: &Path, opened: Option<&Path>) -> bool {
    let mut message = OPEN.to_vec();
    if let Some(path) = opened.and_then(Path::to_str) {
        message.extend_from_slice(path.as_bytes());
    }
    (0..KNOCKS).any(|attempt| {
        if attempt > 0 {
            thread::sleep(PAUSE);
        }
        ring(folder, &message).is_ok()
    })
}

pub fn listen(folder: &Path, open: impl Fn(Option<PathBuf>) + Send + 'static) {
    let folder = folder.to_owned();
    let spawned = thread::Builder::new()
        .name(THREAD_NAME.to_owned())
        .spawn(move || serve(&folder, open));
    if let Err(error) = spawned {
        log_error!("other launches cannot reach this window: {error}");
    }
}

fn serve(folder: &Path, open: impl Fn(Option<PathBuf>)) {
    let mut entrance = match enter(folder) {
        Ok(entrance) => entrance,
        Err(error) => return log_error!("other launches cannot reach this window: {error}"),
    };
    loop {
        match entrance.next_visitor() {
            Ok(visitor) => {
                if let Some(opened) = heard(visitor) {
                    open(opened);
                }
            }
            Err(error) => return log_error!("other launches stop reaching this window: {error}"),
        }
    }
}

fn enter(folder: &Path) -> io::Result<Entrance> {
    let mut entered = Entrance::open(folder);
    for _ in 1..ATTEMPTS {
        if entered.is_ok() {
            break;
        }
        thread::sleep(PAUSE);
        entered = Entrance::open(folder);
    }
    entered
}

fn heard(visitor: impl Read) -> Option<Option<PathBuf>> {
    let mut message = Vec::new();
    visitor.take(LONGEST).read_to_end(&mut message).ok()?;
    let path = std::str::from_utf8(message.strip_prefix(OPEN)?).ok()?;
    Some((!path.is_empty()).then(|| PathBuf::from(path)))
}
