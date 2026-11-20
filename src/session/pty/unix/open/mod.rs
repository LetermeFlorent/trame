use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

#[cfg(test)]
mod tests;

use super::window_size;
use crate::session::pty::PtySize;

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
const OPEN_FLAGS: libc::c_int = libc::O_RDWR | libc::O_NOCTTY | libc::O_CLOEXEC;

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
pub(super) fn open_pty(size: PtySize) -> io::Result<(OwnedFd, OwnedFd)> {
    let master = open_master()?;
    let slave = open_slave(&master)?;
    let size = window_size(size);
    if unsafe { libc::ioctl(slave.as_raw_fd(), libc::TIOCSWINSZ as _, &size) } == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok((master, slave))
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
fn open_master() -> io::Result<OwnedFd> {
    open_master_with(
        |flags| open_fd(unsafe { libc::posix_openpt(flags) }),
        || open_fd(unsafe { libc::open(c"/dev/ptmx".as_ptr(), OPEN_FLAGS) }),
    )
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
fn open_master_with(
    mut open_pt: impl FnMut(libc::c_int) -> io::Result<OwnedFd>,
    open_ptmx: impl FnOnce() -> io::Result<OwnedFd>,
) -> io::Result<OwnedFd> {
    let master = match open_pt(OPEN_FLAGS) {
        Ok(fd) => fd,
        Err(err) if err.raw_os_error() == Some(libc::EINVAL) => match open_ptmx() {
            Ok(fd) => fd,
            Err(_) => {
                let fd = open_pt(libc::O_RDWR | libc::O_NOCTTY)?;
                set_close_on_exec(&fd)?;
                fd
            }
        },
        Err(err) => return Err(err),
    };
    #[cfg(target_vendor = "apple")]
    if unsafe { libc::grantpt(master.as_raw_fd()) } == -1 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::unlockpt(master.as_raw_fd()) } == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(master)
}

#[cfg(target_os = "linux")]
fn open_slave(master: &OwnedFd) -> io::Result<OwnedFd> {
    let slave = unsafe { libc::ioctl(master.as_raw_fd(), libc::TIOCGPTPEER, OPEN_FLAGS) };
    if slave != -1 {
        return Ok(unsafe { OwnedFd::from_raw_fd(slave) });
    }
    let error = io::Error::last_os_error();
    if matches!(
        error.raw_os_error(),
        Some(libc::ENOTTY | libc::ENOSYS | libc::EPERM | libc::EINVAL | libc::EOPNOTSUPP)
    ) {
        return open_slave_by_path(master);
    }
    Err(error)
}

#[cfg(any(target_os = "android", target_vendor = "apple"))]
fn open_slave(master: &OwnedFd) -> io::Result<OwnedFd> {
    open_slave_by_path(master)
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
fn open_slave_by_path(master: &OwnedFd) -> io::Result<OwnedFd> {
    let mut name = [0 as libc::c_char; 128];
    slave_name(master, &mut name)?;
    open_fd(unsafe { libc::open(name.as_ptr(), OPEN_FLAGS) })
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
fn open_fd(fd: libc::c_int) -> io::Result<OwnedFd> {
    if fd == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn slave_name(master: &OwnedFd, name: &mut [libc::c_char; 128]) -> io::Result<()> {
    let error = unsafe { libc::ptsname_r(master.as_raw_fd(), name.as_mut_ptr(), name.len()) };
    if error != 0 {
        return Err(io::Error::from_raw_os_error(error));
    }
    Ok(())
}

#[cfg(target_vendor = "apple")]
fn slave_name(master: &OwnedFd, name: &mut [libc::c_char; 128]) -> io::Result<()> {
    if unsafe {
        libc::ioctl(
            master.as_raw_fd(),
            libc::TIOCPTYGNAME as _,
            name.as_mut_ptr(),
        )
    } == -1
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
pub(super) fn open_pty(size: PtySize) -> io::Result<(OwnedFd, OwnedFd)> {
    let mut master: libc::c_int = -1;
    let mut slave: libc::c_int = -1;
    let mut size = window_size(size);
    let opened = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
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
