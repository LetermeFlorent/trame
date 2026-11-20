#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
use std::io;
use std::os::fd::AsRawFd;

use super::open_pty;
use crate::session::pty::PtySize;

fn close_on_exec(fd: &impl AsRawFd) -> bool {
    let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFD) };
    flags != -1 && flags & libc::FD_CLOEXEC != 0
}

#[test]
fn master_and_slave_open_with_close_on_exec() {
    let size = PtySize {
        rows: 30,
        cols: 100,
    };
    let (master, slave) = open_pty(size).expect("a pty");
    assert!(close_on_exec(&master));
    assert!(close_on_exec(&slave));

    let mut actual = unsafe { std::mem::zeroed::<libc::winsize>() };
    assert_eq!(
        unsafe { libc::ioctl(master.as_raw_fd(), libc::TIOCGWINSZ as _, &raw mut actual) },
        0
    );
    assert_eq!((actual.ws_row, actual.ws_col), (size.rows, size.cols));
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn slave_opened_by_path_also_sets_close_on_exec() {
    let master = super::open_master().expect("master");
    let slave = super::open_slave_by_path(&master).expect("slave via path");
    assert!(close_on_exec(&master));
    assert!(close_on_exec(&slave));
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn einval_falls_back_to_ptmx_with_close_on_exec() {
    let mut pt_flags = Vec::new();
    let master = super::open_master_with(
        |flags| {
            pt_flags.push(flags);
            Err(io::Error::from_raw_os_error(libc::EINVAL))
        },
        || super::open_fd(unsafe { libc::open(c"/dev/ptmx".as_ptr(), super::OPEN_FLAGS) }),
    )
    .expect("ptmx fallback");
    assert_eq!(pt_flags, vec![super::OPEN_FLAGS]);
    assert!(close_on_exec(&master));
    let slave = super::open_slave(&master).expect("slave");
    assert!(close_on_exec(&slave));
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn ptmx_failure_falls_back_to_posix_openpt_without_cloexec() {
    let mut pt_flags = Vec::new();
    let master = super::open_master_with(
        |flags| {
            pt_flags.push(flags);
            if flags & libc::O_CLOEXEC != 0 {
                Err(io::Error::from_raw_os_error(libc::EINVAL))
            } else {
                super::open_fd(unsafe { libc::posix_openpt(flags) })
            }
        },
        || Err(io::Error::from_raw_os_error(libc::ENOENT)),
    )
    .expect("posix_openpt fallback");
    assert_eq!(
        pt_flags,
        vec![super::OPEN_FLAGS, libc::O_RDWR | libc::O_NOCTTY]
    );
    assert!(close_on_exec(&master));
    let slave = super::open_slave(&master).expect("slave");
    assert!(close_on_exec(&slave));
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn non_einval_error_skips_fallbacks() {
    let mut pt_calls = 0;
    let mut ptmx_called = false;
    let result = super::open_master_with(
        |_| {
            pt_calls += 1;
            Err(io::Error::from_raw_os_error(libc::EACCES))
        },
        || {
            ptmx_called = true;
            Err(io::Error::from_raw_os_error(libc::ENOENT))
        },
    );
    let err = result.expect_err("non-EINVAL error");
    assert_eq!(err.raw_os_error(), Some(libc::EACCES));
    assert_eq!(pt_calls, 1);
    assert!(!ptmx_called);
}
