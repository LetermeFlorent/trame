use std::fs::File;
use std::io::{self, ErrorKind, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(test)]
mod tests;

#[cfg(feature = "dump")]
use super::dump::Dump;
use super::sync::SyncHold;
use super::{Notifier, SessionEvent, SessionId, lock};
use crate::secret::wipe;
use crate::session::pty::{self, Child};
use crate::term::{Modes, Terminal};

const READ_BUFFER: usize = 64 * 1024;
const BACKLOG: usize = 1024;
const STACK_SIZE: usize = 256 * 1024;

pub struct Reader {
    pub id: SessionId,
    pub source: File,
    pub shell: u32,
    pub terminal: Arc<Mutex<Terminal>>,
    pub input: Sender<Vec<u8>>,
    pub pending: Arc<AtomicBool>,
    pub notify: Notifier,
}

pub fn spawn_reader(reader: Reader) -> io::Result<()> {
    spawn("trame-read", move || read_loop(reader))
}

pub fn spawn_writer(mut sink: File, queue: Receiver<Vec<u8>>) -> io::Result<()> {
    spawn("trame-write", move || {
        for mut bytes in queue {
            let written = sink.write_all(&bytes);
            wipe(&mut bytes);
            if written.is_err() {
                break;
            }
        }
    })
}

pub fn spawn_waiter(id: SessionId, child: Child, notify: Notifier) -> io::Result<()> {
    spawn("trame-wait", move || {
        child.wait();
        notify(SessionEvent::Exited(id));
    })
}

fn read_loop(reader: Reader) {
    let mut buffer = vec![0; READ_BUFFER];
    let mut hold = SyncHold::new(reader.id, reader.pending.clone(), reader.notify.clone());
    #[cfg(feature = "dump")]
    let mut dump = Dump::open(reader.id);
    let source = &reader.source;
    while let Some(count) = read_batch(source, &mut buffer, || pty::waiting_bytes(source)) {
        let shell_in_front = pty::shell_in_foreground(source, reader.shell);
        #[cfg(feature = "dump")]
        let working;
        let (responses, synchronized) = {
            let mut terminal = lock(&reader.terminal);
            terminal.feed(&buffer[..count]);
            let screen = terminal.screen_mut();
            screen.shell_in_front(shell_in_front);
            #[cfg(feature = "dump")]
            {
                working = screen.program_working();
            }
            if shell_in_front {
                screen.forget_mouse();
            }
            let synchronized = screen.modes().contains(Modes::SYNC_UPDATE);
            (screen.take_responses(), synchronized)
        };
        #[cfg(feature = "dump")]
        dump.record(&buffer[..count], working);
        if !responses.is_empty() && reader.input.send(responses).is_err() {
            break;
        }
        if !hold.holds(synchronized) {
            wake_output(reader.id, &reader.pending, &reader.notify);
        }
    }
    wake_output(reader.id, &reader.pending, &reader.notify);
}

fn read_batch(
    source: &File,
    buffer: &mut [u8],
    mut waiting: impl FnMut() -> usize,
) -> Option<usize> {
    let mut count = read_some(source, buffer)?;
    while count >= BACKLOG && count < buffer.len() && waiting() > 0 {
        let Some(more) = read_some(source, &mut buffer[count..]) else {
            break;
        };
        count += more;
    }
    Some(count)
}

fn read_some(mut source: &File, buffer: &mut [u8]) -> Option<usize> {
    loop {
        match source.read(buffer) {
            Ok(0) => return None,
            Ok(count) => return Some(count),
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(_) => return None,
        }
    }
}

pub(super) fn wake_output(id: SessionId, pending: &AtomicBool, notify: &Notifier) {
    if !pending.swap(true, Ordering::AcqRel) {
        notify(SessionEvent::Output(id));
    }
}

pub(super) fn spawn(name: &str, work: impl FnOnce() + Send + 'static) -> io::Result<()> {
    thread::Builder::new()
        .name(name.into())
        .stack_size(STACK_SIZE)
        .spawn(work)
        .map(drop)
}
