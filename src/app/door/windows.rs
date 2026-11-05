use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::path::Path;
use std::ptr;

use trame::files::fingerprint;
use windows_sys::Win32::Foundation::{
    ERROR_NO_DATA, ERROR_PIPE_CONNECTED, GetLastError, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_INBOUND};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
    PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{ASFW_ANY, AllowSetForegroundWindow};

const PIPE_PREFIX: &str = r"\\.\pipe\trame-";
const BUFFER_SIZE: u32 = 64;

pub struct Entrance {
    name: Vec<u16>,
    waiting: File,
}

impl Entrance {
    pub fn open(folder: &Path) -> io::Result<Self> {
        let name = wide(&pipe_name(folder));
        let waiting = instance(&name, FILE_FLAG_FIRST_PIPE_INSTANCE)?;
        Ok(Self { name, waiting })
    }

    pub fn next_visitor(&mut self) -> io::Result<File> {
        let connected = unsafe { ConnectNamedPipe(self.waiting.as_raw_handle(), ptr::null_mut()) };
        if connected == 0
            && !matches!(
                unsafe { GetLastError() },
                ERROR_PIPE_CONNECTED | ERROR_NO_DATA
            )
        {
            return Err(io::Error::last_os_error());
        }
        let next = instance(&self.name, 0)?;
        Ok(mem::replace(&mut self.waiting, next))
    }
}

pub fn ring(folder: &Path, message: &[u8]) -> io::Result<()> {
    let mut pipe = OpenOptions::new().write(true).open(pipe_name(folder))?;
    unsafe { AllowSetForegroundWindow(ASFW_ANY) };
    pipe.write_all(message)?;
    pipe.sync_all()
}

fn instance(name: &[u16], first: u32) -> io::Result<File> {
    let handle = unsafe {
        CreateNamedPipeW(
            name.as_ptr(),
            PIPE_ACCESS_INBOUND | first,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            PIPE_UNLIMITED_INSTANCES,
            0,
            BUFFER_SIZE,
            0,
            ptr::null(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_handle(handle) })
}

fn pipe_name(folder: &Path) -> String {
    let path = folder.as_os_str().to_string_lossy().to_lowercase();
    format!("{PIPE_PREFIX}{:016x}", fingerprint(path.as_bytes()))
}

fn wide(text: &str) -> Vec<u16> {
    std::ffi::OsStr::new(text)
        .encode_wide()
        .chain([0])
        .collect()
}
