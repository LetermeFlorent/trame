use std::collections::{HashMap, VecDeque};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};

use windows_sys::Win32::Foundation::{FILETIME, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, GetProcessTimes, OpenProcess,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
};

pub struct Member {
    pub id: u32,
    pub parent: u32,
    pub born: u64,
    pub handle: OwnedHandle,
}

pub fn descendants() -> Vec<Member> {
    let pairs = snapshot();
    let own = unsafe { GetCurrentProcessId() };
    let Some(own_born) = born_of(unsafe { GetCurrentProcess() }) else {
        return Vec::new();
    };
    let mut born = HashMap::from([(own, own_born)]);
    let mut queue = VecDeque::from([own]);
    let mut members = Vec::new();
    while let Some(parent) = queue.pop_front() {
        let parent_born = born[&parent];
        for &(id, _) in pairs.iter().filter(|(_, of)| *of == parent) {
            if born.contains_key(&id) {
                continue;
            }
            let Some(handle) = open(id) else {
                continue;
            };
            let Some(child_born) = born_of(handle.as_raw_handle()).filter(|at| *at >= parent_born)
            else {
                continue;
            };
            born.insert(id, child_born);
            queue.push_back(id);
            members.push(Member {
                id,
                parent,
                born: child_born,
                handle,
            });
        }
    }
    members
}

pub fn time_of(time: FILETIME) -> u64 {
    (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)
}

fn snapshot() -> Vec<(u32, u32)> {
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return Vec::new();
    }
    let snapshot = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut pairs = Vec::new();
    let mut found = unsafe { Process32FirstW(snapshot.as_raw_handle(), &mut entry) } != 0;
    while found {
        pairs.push((entry.th32ProcessID, entry.th32ParentProcessID));
        found = unsafe { Process32NextW(snapshot.as_raw_handle(), &mut entry) } != 0;
    }
    pairs
}

fn open(id: u32) -> Option<OwnedHandle> {
    let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION;
    let raw = unsafe { OpenProcess(access, 0, id) };
    (!raw.is_null()).then(|| unsafe { OwnedHandle::from_raw_handle(raw) })
}

fn born_of(process: HANDLE) -> Option<u64> {
    let mut times = [FILETIME::default(); 4];
    let [created, exited, kernel, user] = &mut times;
    let done = unsafe { GetProcessTimes(process, created, exited, kernel, user) } != 0;
    done.then(|| time_of(times[0]))
}
