mod cwd;
mod foreground;

pub(super) use foreground::shell_in_foreground;

use std::fs::File;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::{DUMP_VARIABLE, PtyOptions, PtySize, Spawned, terminal_env};

const TERM: &str = "xterm-256color";

pub(super) struct Pty {
    master: OwnedFd,
    pid: libc::pid_t,
    exited: Arc<AtomicBool>,
}

impl Pty {
    pub(super) fn resize(&self, size: PtySize) -> io::Result<()> {
        let size = window_size(size);
        if unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ as _, &size) } == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub(super) fn process_id(&self) -> u32 {
        self.pid as u32
    }

    pub(super) fn cwd(&self) -> Option<PathBuf> {
        cwd::process_cwd(self.pid)
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        if !self.exited.load(Ordering::Acquire) {
            unsafe { libc::kill(self.pid, libc::SIGHUP) };
        }
    }
}

pub(super) struct Child {
    child: std::process::Child,
    exited: Arc<AtomicBool>,
}

impl Child {
    pub(super) fn wait(mut self) -> Option<i32> {
        let status = self.child.wait().ok();
        self.exited.store(true, Ordering::Release);
        status.and_then(|status| status.code())
    }
}

pub(super) fn spawn(options: &PtyOptions) -> io::Result<Spawned> {
    let (master, slave) = open_pty(options.size)?;
    let mut command = Command::new(&options.shell.program);
    command
        .args(&options.shell.args)
        .stdin(Stdio::from(slave.try_clone()?))
        .stdout(Stdio::from(slave.try_clone()?))
        .stderr(Stdio::from(slave))
        .env("TERM", TERM)
        .envs(terminal_env())
        .envs(options.shell.env.iter().cloned())
        .env_remove(DUMP_VARIABLE);
    if let Some(cwd) = options.cwd {
        command.current_dir(cwd);
    }
    unsafe { command.pre_exec(attach_controlling_terminal) };
    let child = command.spawn()?;
    let exited = Arc::new(AtomicBool::new(false));
    let reader = File::from(master.try_clone()?);
    let writer = File::from(master.try_clone()?);
    Ok(Spawned {
        pty: super::Pty(Pty {
            master,
            pid: child.id() as libc::pid_t,
            exited: Arc::clone(&exited),
        }),
        reader,
        writer,
        child: super::Child(Child { child, exited }),
    })
}

fn attach_controlling_terminal() -> io::Result<()> {
    if unsafe { libc::setsid() } == -1 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::ioctl(0, libc::TIOCSCTTY as _, 0) } == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn open_pty(size: PtySize) -> io::Result<(OwnedFd, OwnedFd)> {
    let mut master: RawFd = -1;
    let mut slave: RawFd = -1;
    let mut size = window_size(size);
    let opened = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            null_mut(),
            null_mut(),
            &raw mut size,
        )
    };
    if opened == -1 {
        return Err(io::Error::last_os_error());
    }
    let (master, slave) = unsafe { (OwnedFd::from_raw_fd(master), OwnedFd::from_raw_fd(slave)) };
    set_close_on_exec(&master)?;
    set_close_on_exec(&slave)?;
    Ok((master, slave))
}

fn set_close_on_exec(fd: &OwnedFd) -> io::Result<()> {
    let fd = fd.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags == -1 || unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn window_size(size: PtySize) -> libc::winsize {
    libc::winsize {
        ws_row: size.rows,
        ws_col: size.cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    }
}
