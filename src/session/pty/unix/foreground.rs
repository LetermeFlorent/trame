use std::fs::File;
use std::os::fd::AsRawFd;

pub(in crate::session::pty) fn shell_in_foreground(terminal: &File, shell: u32) -> bool {
    let group = unsafe { libc::tcgetpgrp(terminal.as_raw_fd()) };
    group == shell as libc::pid_t
}
