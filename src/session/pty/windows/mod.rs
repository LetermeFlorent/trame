mod command_line;
mod pipes;
mod process;

use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::os::windows::io::{AsRawHandle, OwnedHandle};
use std::path::PathBuf;

use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::System::Console::{
    COORD, ClosePseudoConsole, CreatePseudoConsole, HPCON, PSEUDOCONSOLE_INHERIT_CURSOR,
    ResizePseudoConsole,
};
use windows_sys::Win32::System::Threading::{GetExitCodeProcess, INFINITE, WaitForSingleObject};

use super::{PtyOptions, PtySize, Spawned, terminal_env};

struct Console(HPCON);

impl Drop for Console {
    fn drop(&mut self) {
        unsafe { ClosePseudoConsole(self.0) };
    }
}

pub(super) struct Pty {
    console: Console,
    process_id: u32,
}

impl Pty {
    pub(super) fn resize(&self, size: PtySize) -> io::Result<()> {
        check(
            unsafe { ResizePseudoConsole(self.console.0, coord(size)) },
            "ResizePseudoConsole",
        )
    }

    pub(super) fn process_id(&self) -> u32 {
        self.process_id
    }

    pub(super) fn cwd(&self) -> Option<PathBuf> {
        None
    }
}

pub(super) fn shell_in_foreground(_terminal: &File, _shell: u32) -> bool {
    false
}

pub(super) struct Child {
    process: OwnedHandle,
}

impl Child {
    pub(super) fn wait(self) -> Option<i32> {
        let handle = self.process.as_raw_handle();
        if unsafe { WaitForSingleObject(handle, INFINITE) } != WAIT_OBJECT_0 {
            return None;
        }
        let mut code = 0;
        if unsafe { GetExitCodeProcess(handle, &mut code) } == 0 {
            return None;
        }
        Some(code as i32)
    }
}

pub(super) fn spawn(options: &PtyOptions) -> io::Result<Spawned> {
    let input = pipes::create_pipe()?;
    let output = pipes::create_pipe()?;
    let console = create_console(options.size, &input.read, &output.write)?;
    drop(input.read);
    drop(output.write);
    let shell = options.shell;
    let mut line = command_line::command_line(shell.program.as_os_str(), &shell.args);
    let environment = command_line::environment_block(&child_env(options));
    let cwd = options.cwd.map(|path| command_line::wide(path.as_os_str()));
    let process = process::create_process(console.0, &mut line, &environment, cwd.as_deref())?;
    Ok(Spawned {
        pty: super::Pty(Pty {
            console,
            process_id: process.id,
        }),
        reader: File::from(output.read),
        writer: File::from(input.write),
        child: super::Child(Child {
            process: process.handle,
        }),
    })
}

fn create_console(size: PtySize, input: &OwnedHandle, output: &OwnedHandle) -> io::Result<Console> {
    let mut console: HPCON = 0;
    let result = unsafe {
        CreatePseudoConsole(
            coord(size),
            input.as_raw_handle(),
            output.as_raw_handle(),
            PSEUDOCONSOLE_INHERIT_CURSOR,
            &mut console,
        )
    };
    check(result, "CreatePseudoConsole")?;
    Ok(Console(console))
}

fn child_env(options: &PtyOptions) -> Vec<(OsString, OsString)> {
    terminal_env()
        .into_iter()
        .map(|(key, value)| (OsString::from(key), OsString::from(value)))
        .chain(options.shell.env.iter().cloned())
        .collect()
}

fn coord(size: PtySize) -> COORD {
    COORD {
        X: size.cols as i16,
        Y: size.rows as i16,
    }
}

fn check(result: i32, call: &str) -> io::Result<()> {
    if result < 0 {
        return Err(io::Error::other(format!(
            "{call} failed with {result:#010x}"
        )));
    }
    Ok(())
}
