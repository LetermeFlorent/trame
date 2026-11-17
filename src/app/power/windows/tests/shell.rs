use std::io::Write;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{HANDLE, MAX_PATH};
use windows_sys::Win32::System::Threading::{
    BELOW_NORMAL_PRIORITY_CLASS, GetCurrentProcess, GetPriorityClass, OpenProcess,
    PROCESS_NAME_WIN32, PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_TERMINATE,
    QueryFullProcessImageNameW, TerminateProcess,
};

use super::super::tree::{Member, descendants};
use super::super::{eco, speed_of, throttle};

pub(super) type State = (u32, u32);

pub(super) const ECO: u32 = PROCESS_POWER_THROTTLING_EXECUTION_SPEED;
pub(super) const SLOWED: State = (ECO, ECO);
pub(super) const SHELL_PINGS: &str = "30";
const PING_IMAGE: &str = r"\PING.EXE";
const CHILD_WAIT: Duration = Duration::from_secs(5);
const CHILD_POLL: Duration = Duration::from_millis(10);

pub(super) struct Shell(Child);

impl Shell {
    pub(super) fn start() -> Self {
        Self::at(BELOW_NORMAL_PRIORITY_CLASS)
    }

    pub(super) fn at(class: u32) -> Self {
        let child = Command::new("ping")
            .args(["-n", SHELL_PINGS, "127.0.0.1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .creation_flags(class)
            .spawn()
            .expect("ping starts");
        Self(child)
    }

    pub(super) fn choosing_efficiency_mode() -> Self {
        let shell = Self::start();
        let handle = shell.0.as_raw_handle();
        throttle(handle, eco(true), unsafe { GetPriorityClass(handle) });
        shell
    }

    pub(super) fn reading_commands() -> Self {
        let child = Command::new("cmd")
            .args(["/q", "/k"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .creation_flags(BELOW_NORMAL_PRIORITY_CLASS)
            .spawn()
            .expect("cmd starts");
        Self(child)
    }

    pub(super) fn run(&mut self, line: &str) {
        let input = self.0.stdin.as_mut().expect("cmd input");
        writeln!(input, "{line}").expect("cmd reads");
    }

    pub(super) fn id(&self) -> u32 {
        self.0.id()
    }

    pub(super) fn state(&self) -> State {
        state_of(self.0.as_raw_handle()).expect("power state readable")
    }

    pub(super) fn class(&self) -> u32 {
        class_of(self.0.as_raw_handle())
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub(super) struct Orphan(pub(super) Member);

impl Orphan {
    pub(super) fn state(&self) -> State {
        state_of(self.0.handle.as_raw_handle()).expect("power state readable")
    }
}

impl Drop for Orphan {
    fn drop(&mut self) {
        let raw = unsafe { OpenProcess(PROCESS_TERMINATE, 0, self.0.id) };
        if !raw.is_null() {
            let process = unsafe { OwnedHandle::from_raw_handle(raw) };
            unsafe { TerminateProcess(process.as_raw_handle(), 1) };
        }
    }
}

pub(super) fn class_of(process: HANDLE) -> u32 {
    unsafe { GetPriorityClass(process) }
}

pub(super) fn first_child_of(parent: u32) -> Member {
    let deadline = Instant::now() + CHILD_WAIT;
    loop {
        if let Some(child) = descendants([])
            .1
            .into_iter()
            .find(|member| member.parent == parent && is_ping(member.handle.as_raw_handle()))
        {
            return child;
        }
        assert!(Instant::now() < deadline, "{parent} started no ping");
        sleep(CHILD_POLL);
    }
}

fn is_ping(process: HANDLE) -> bool {
    let mut path = [0; MAX_PATH as usize];
    let mut length = path.len() as u32;
    let read = unsafe {
        QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, path.as_mut_ptr(), &mut length)
    } != 0;
    let image = read.then(|| String::from_utf16_lossy(&path[..length as usize]));
    image.is_some_and(|image| image.to_ascii_uppercase().ends_with(PING_IMAGE))
}

pub(super) fn state_of(process: HANDLE) -> Option<State> {
    speed_of(process).map(|speed| (speed.ControlMask, speed.StateMask))
}

pub(super) fn unsupported() -> bool {
    let missing = state_of(unsafe { GetCurrentProcess() }).is_none();
    if missing {
        eprintln!("skipped: GetProcessInformation(ProcessPowerThrottling) is not supported here");
    }
    missing
}
