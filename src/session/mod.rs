mod dump;
pub mod pty;
mod sync;
mod threads;
mod title;
mod transcript;

use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;

use crate::term::{DEFAULT_HISTORY, ReportedColors, Terminal};
use pty::{Pty, PtyOptions, PtySize, ShellCommand};
pub use title::starting_label;
pub use transcript::{
    MAX_TRANSCRIPT_BYTES, TRANSCRIPT_SAVE_INTERVAL, Transcripts, transcript_of, transcript_within,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionEvent {
    Output(SessionId),
    Exited(SessionId),
}

pub type Notifier = Arc<dyn Fn(SessionEvent) + Send + Sync>;

pub struct SessionOptions<'a> {
    pub shell: &'a ShellCommand,
    pub cwd: Option<&'a Path>,
    pub rows: usize,
    pub cols: usize,
    pub colors: ReportedColors,
    pub transcript: &'a [u8],
}

pub struct Session {
    id: SessionId,
    terminal: Arc<Mutex<Terminal>>,
    input: Sender<Vec<u8>>,
    pending: Arc<AtomicBool>,
    pty: Option<Pty>,
    size: (usize, usize),
    shell: String,
}

impl Session {
    pub fn spawn(id: SessionId, options: &SessionOptions, notify: Notifier) -> io::Result<Self> {
        let rows = options.rows.max(1);
        let cols = options.cols.max(1);
        let spawned = pty::spawn(&PtyOptions {
            shell: options.shell,
            cwd: options.cwd,
            size: PtySize::new(rows, cols),
        })?;
        let mut terminal = Terminal::new(rows, cols, DEFAULT_HISTORY);
        terminal.screen_mut().set_reported_colors(options.colors);
        terminal.feed(options.transcript);
        let terminal = Arc::new(Mutex::new(terminal));
        let (input, queue) = mpsc::channel();
        let pending = Arc::new(AtomicBool::new(false));
        threads::spawn_writer(spawned.writer, queue)?;
        threads::spawn_reader(threads::Reader {
            id,
            source: spawned.reader,
            shell: spawned.pty.process_id(),
            terminal: terminal.clone(),
            input: input.clone(),
            pending: pending.clone(),
            notify: notify.clone(),
        })?;
        threads::spawn_waiter(id, spawned.child, notify)?;
        Ok(Self {
            id,
            terminal,
            input,
            pending,
            pty: Some(spawned.pty),
            size: (rows, cols),
            shell: title::shell_name(&options.shell.program),
        })
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn terminal(&self) -> MutexGuard<'_, Terminal> {
        lock(&self.terminal)
    }

    pub fn write(&self, bytes: impl Into<Vec<u8>>) {
        let bytes = bytes.into();
        if !bytes.is_empty() {
            let _ = self.input.send(bytes);
        }
    }

    pub fn take_output(&self) -> bool {
        self.pending.swap(false, Ordering::AcqRel)
    }

    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        let size = (rows.max(1), cols.max(1));
        if size == self.size {
            return;
        }
        self.size = size;
        self.terminal().screen_mut().resize(size.0, size.1);
        if let Some(pty) = &self.pty
            && let Err(error) = pty.resize(PtySize::new(size.0, size.1))
        {
            eprintln!("trame: resizing the terminal failed: {error}");
        }
    }

    pub fn shell_id(&self) -> Option<u32> {
        self.pty.as_ref().map(Pty::process_id)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(pty) = self.pty.take() {
            let _ = thread::Builder::new()
                .name("trame-close".into())
                .spawn(move || drop(pty));
        }
    }
}

pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
