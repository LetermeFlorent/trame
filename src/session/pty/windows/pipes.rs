use std::fs::File;
use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::ptr::{null, null_mut};

use windows_sys::Win32::System::Pipes::{CreatePipe, PeekNamedPipe};

pub(super) struct Pipe {
    pub read: OwnedHandle,
    pub write: OwnedHandle,
}

pub(super) fn create_pipe() -> io::Result<Pipe> {
    let mut read = null_mut();
    let mut write = null_mut();
    if unsafe { CreatePipe(&mut read, &mut write, null(), 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    unsafe {
        Ok(Pipe {
            read: OwnedHandle::from_raw_handle(read),
            write: OwnedHandle::from_raw_handle(write),
        })
    }
}

pub(in crate::session::pty) fn waiting_bytes(pipe: &File) -> usize {
    let mut waiting = 0u32;
    let peeked = unsafe {
        PeekNamedPipe(
            pipe.as_raw_handle(),
            null_mut(),
            0,
            null_mut(),
            &raw mut waiting,
            null_mut(),
        )
    };
    if peeked == 0 { 0 } else { waiting as usize }
}
