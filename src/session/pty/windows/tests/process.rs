use std::fs;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{FILETIME, HANDLE, INVALID_HANDLE_VALUE, WAIT_TIMEOUT};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    PROCESS_TERMINATE, TerminateProcess, WaitForSingleObject,
};

pub(super) const WAIT_LIMIT: Duration = Duration::from_secs(10);
const POLL_STEP: Duration = Duration::from_millis(20);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Target {
    pub pid: u32,
    pub born: u64,
}

pub(super) struct Cleanup {
    targets: Vec<Target>,
    paths: Vec<PathBuf>,
}

impl Cleanup {
    pub(super) fn new() -> Self {
        Self {
            targets: Vec::new(),
            paths: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, target: Target) {
        if target.pid != 0 {
            self.targets.push(target);
        }
    }

    pub(super) fn mark_dead(&mut self, dead: &[Target]) {
        self.targets.retain(|target| !dead.contains(target));
    }

    pub(super) fn push_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        for &target in &self.targets {
            kill_if_alive(target);
        }
        for path in &self.paths {
            let _ = fs::remove_file(path);
        }
    }
}

pub(super) fn snapshot() -> Vec<(u32, u32, String)> {
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return Vec::new();
    }
    let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut rows = Vec::new();
    let mut more = unsafe { Process32FirstW(handle.as_raw_handle(), &mut entry) } != 0;
    while more {
        let end = entry
            .szExeFile
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(entry.szExeFile.len());
        let name = String::from_utf16_lossy(&entry.szExeFile[..end]).to_ascii_lowercase();
        rows.push((entry.th32ProcessID, entry.th32ParentProcessID, name));
        more = unsafe { Process32NextW(handle.as_raw_handle(), &mut entry) } != 0;
    }
    rows
}

fn open_process(pid: u32, access: u32) -> Option<OwnedHandle> {
    let raw = unsafe { OpenProcess(access, 0, pid) };
    (!raw.is_null()).then(|| unsafe { OwnedHandle::from_raw_handle(raw) })
}

pub(super) fn born_of(handle: HANDLE) -> Option<u64> {
    let mut times = [FILETIME::default(); 4];
    let [created, exited, kernel, user] = &mut times;
    let ok = unsafe { GetProcessTimes(handle, created, exited, kernel, user) } != 0;
    ok.then(|| (u64::from(times[0].dwHighDateTime) << 32) | u64::from(times[0].dwLowDateTime))
}

pub(super) fn target_of(pid: u32) -> Option<Target> {
    let handle = open_process(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
    let born = born_of(handle.as_raw_handle())?;
    Some(Target { pid, born })
}

fn open_live(target: Target, extra: u32) -> Option<OwnedHandle> {
    let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE | extra;
    let handle = open_process(target.pid, access)?;
    let raw = handle.as_raw_handle();
    let alive =
        born_of(raw) == Some(target.born) && unsafe { WaitForSingleObject(raw, 0) == WAIT_TIMEOUT };
    alive.then_some(handle)
}

pub(super) fn is_alive(target: Target) -> bool {
    open_live(target, 0).is_some()
}

pub(super) fn kill_if_alive(target: Target) {
    if let Some(handle) = open_live(target, PROCESS_TERMINATE) {
        unsafe { TerminateProcess(handle.as_raw_handle(), 1) };
    }
}

pub(super) fn wait_until(timeout: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if condition() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        sleep(POLL_STEP);
    }
}

pub(super) fn conhost_child(parent: u32) -> Option<Target> {
    snapshot()
        .into_iter()
        .find(|(_, ppid, name)| *ppid == parent && name == "conhost.exe")
        .and_then(|(pid, _, _)| target_of(pid))
}
