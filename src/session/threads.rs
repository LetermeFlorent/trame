use std::fs::File;
use std::hint::black_box;
use std::io::{self, ErrorKind, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use super::dump::Dump;
use super::sync::SyncHold;
use super::{Notifier, SessionEvent, SessionId, lock};
use crate::session::pty::{self, Child};
use crate::term::{Modes, Terminal};

const READ_BUFFER: usize = 64 * 1024;
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
            bytes.fill(0);
            black_box(&bytes);
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

fn read_loop(mut reader: Reader) {
    let mut buffer = vec![0; READ_BUFFER];
    let mut hold = SyncHold::new(reader.id, reader.pending.clone(), reader.notify.clone());
    let mut dump = Dump::open(reader.id);
    loop {
        let count = match reader.source.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => count,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        dump.record(&buffer[..count]);
        let shell_in_front = pty::shell_in_foreground(&reader.source, reader.shell);
        let (responses, synchronized) = {
            let mut terminal = lock(&reader.terminal);
            terminal.feed(&buffer[..count]);
            let screen = terminal.screen_mut();
            if shell_in_front {
                screen.forget_mouse();
            }
            let synchronized = screen.modes().contains(Modes::SYNC_UPDATE);
            (screen.take_responses(), synchronized)
        };
        if !responses.is_empty() && reader.input.send(responses).is_err() {
            break;
        }
        if !hold.holds(synchronized) {
            wake_output(reader.id, &reader.pending, &reader.notify);
        }
    }
    wake_output(reader.id, &reader.pending, &reader.notify);
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
