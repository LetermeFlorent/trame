use std::fs::File;
use std::io::{self, ErrorKind, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use super::{Notifier, SessionEvent, SessionId, lock};
use crate::pty::Child;
use crate::term::{Modes, Terminal};

const READ_BUFFER: usize = 64 * 1024;
const STACK_SIZE: usize = 256 * 1024;

pub struct Reader {
    pub id: SessionId,
    pub source: File,
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
        for bytes in queue {
            if sink.write_all(&bytes).is_err() {
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
    loop {
        let count = match reader.source.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => count,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        let (responses, synchronized) = {
            let mut terminal = lock(&reader.terminal);
            terminal.feed(&buffer[..count]);
            let screen = terminal.screen_mut();
            let synchronized = screen.modes().contains(Modes::SYNC_UPDATE);
            (screen.take_responses(), synchronized)
        };
        if !responses.is_empty() && reader.input.send(responses).is_err() {
            break;
        }
        if !synchronized && !reader.pending.swap(true, Ordering::AcqRel) {
            (reader.notify)(SessionEvent::Output(reader.id));
        }
    }
    if !reader.pending.swap(true, Ordering::AcqRel) {
        (reader.notify)(SessionEvent::Output(reader.id));
    }
}

fn spawn(name: &str, work: impl FnOnce() + Send + 'static) -> io::Result<()> {
    thread::Builder::new()
        .name(name.into())
        .stack_size(STACK_SIZE)
        .spawn(work)
        .map(drop)
}
