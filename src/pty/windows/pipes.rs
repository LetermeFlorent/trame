use std::io;
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::ptr::{null, null_mut};

use windows_sys::Win32::System::Pipes::CreatePipe;

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
