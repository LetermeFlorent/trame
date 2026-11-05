#[cfg(test)]
mod tests;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::io::{self, Read};
use std::path::Path;
use std::thread;
use std::time::Duration;

use trame::log_error;

#[cfg(unix)]
use unix::{Entrance, ring};
#[cfg(windows)]
use windows::{Entrance, ring};

const OPEN: &[u8] = b"open\n";
const THREAD_NAME: &str = "trame-door";
const ATTEMPTS: u32 = 20;
const KNOCKS: u32 = 4;
const PAUSE: Duration = Duration::from_millis(250);

pub fn knock(folder: &Path) -> bool {
    (0..KNOCKS).any(|attempt| {
        if attempt > 0 {
            thread::sleep(PAUSE);
        }
        ring(folder, OPEN).is_ok()
    })
}

pub fn listen(folder: &Path, open: impl Fn() + Send + 'static) {
    let folder = folder.to_owned();
    let spawned = thread::Builder::new()
        .name(THREAD_NAME.to_owned())
        .spawn(move || serve(&folder, open));
    if let Err(error) = spawned {
        log_error!("other launches cannot reach this window: {error}");
    }
}

fn serve(folder: &Path, open: impl Fn()) {
    let mut entrance = match enter(folder) {
        Ok(entrance) => entrance,
        Err(error) => return log_error!("other launches cannot reach this window: {error}"),
    };
    loop {
        match entrance.next_visitor() {
            Ok(visitor) => {
                if heard(visitor) {
                    open();
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

fn heard(visitor: impl Read) -> bool {
    let mut message = Vec::with_capacity(OPEN.len());
    let read = visitor.take(OPEN.len() as u64).read_to_end(&mut message);
    read.is_ok() && message == OPEN
}
