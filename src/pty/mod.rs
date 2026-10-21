mod shell;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix as platform;
#[cfg(windows)]
use windows as platform;

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

pub use shell::{ShellCommand, default_shell};

pub const TERM_PROGRAM: &str = "trame";

const MAX_DIMENSION: usize = i16::MAX as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PtySize {
    pub rows: u16,
    pub cols: u16,
}

impl PtySize {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows: rows.clamp(1, MAX_DIMENSION) as u16,
            cols: cols.clamp(1, MAX_DIMENSION) as u16,
        }
    }
}

pub struct PtyOptions<'a> {
    pub shell: &'a ShellCommand,
    pub cwd: Option<&'a Path>,
    pub size: PtySize,
}

pub struct Spawned {
    pub pty: Pty,
    pub reader: File,
    pub writer: File,
    pub child: Child,
}

pub struct Pty(platform::Pty);

impl Pty {
    pub fn resize(&self, size: PtySize) -> io::Result<()> {
        self.0.resize(size)
    }

    pub fn process_id(&self) -> u32 {
        self.0.process_id()
    }

    pub fn cwd(&self) -> Option<PathBuf> {
        self.0.cwd()
    }
}

pub struct Child(platform::Child);

impl Child {
    pub fn wait(self) -> Option<i32> {
        self.0.wait()
    }
}

pub fn spawn(options: &PtyOptions) -> io::Result<Spawned> {
    platform::spawn(options)
}

fn terminal_env() -> [(&'static str, &'static str); 3] {
    [
        ("COLORTERM", "truecolor"),
        ("TERM_PROGRAM", TERM_PROGRAM),
        ("TERM_PROGRAM_VERSION", env!("CARGO_PKG_VERSION")),
    ]
}
