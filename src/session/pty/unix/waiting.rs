use std::fs::File;
use std::os::fd::AsRawFd;

const NO_WAIT: libc::c_int = 0;

pub(in crate::session::pty) fn waiting_bytes(terminal: &File) -> usize {
    let mut asked = libc::pollfd {
        fd: terminal.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let answered = unsafe { libc::poll(&raw mut asked, 1, NO_WAIT) } > 0;
    usize::from(answered && asked.revents & (libc::POLLIN | libc::POLLHUP) != 0)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::os::fd::AsRawFd;

    use super::super::open_pty;
    use super::waiting_bytes;
    use crate::session::pty::PtySize;

    const SIZE: PtySize = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };

    fn raw_pty() -> (File, File) {
        let (master, slave) = open_pty(SIZE).expect("a pty");
        let mut settings = unsafe { std::mem::zeroed::<libc::termios>() };
        assert_eq!(
            unsafe { libc::tcgetattr(slave.as_raw_fd(), &raw mut settings) },
            0
        );
        unsafe { libc::cfmakeraw(&raw mut settings) };
        let set = unsafe { libc::tcsetattr(slave.as_raw_fd(), libc::TCSANOW, &raw const settings) };
        assert_eq!(set, 0);
        (File::from(master), File::from(slave))
    }

    #[test]
    fn typed_input_the_child_has_not_read_is_not_output() {
        let (mut master, _slave) = raw_pty();
        master.write_all(b"typed").expect("room for typed input");
        assert_eq!(waiting_bytes(&master), 0);
    }

    #[test]
    fn a_closed_child_side_never_blocks_a_read() {
        let (master, slave) = raw_pty();
        drop(slave);
        assert_ne!(waiting_bytes(&master), 0);
    }
}
