mod family;
mod tree;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{
    BELOW_NORMAL_PRIORITY_CLASS, GetCurrentProcess, GetCurrentThread, IDLE_PRIORITY_CLASS,
    NORMAL_PRIORITY_CLASS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    ProcessPowerThrottling, SetPriorityClass, SetProcessInformation, SetProcessWorkingSetSize,
    SetThreadPriority, THREAD_PRIORITY_HIGHEST, THREAD_PRIORITY_NORMAL,
};

pub use family::Family;

pub fn set_own(on: bool, hidden: bool) {
    let (class, thread) = match (on, hidden) {
        (true, true) => (IDLE_PRIORITY_CLASS, THREAD_PRIORITY_HIGHEST),
        (true, false) => (BELOW_NORMAL_PRIORITY_CLASS, THREAD_PRIORITY_NORMAL),
        (false, _) => (NORMAL_PRIORITY_CLASS, THREAD_PRIORITY_NORMAL),
    };
    let process = unsafe { GetCurrentProcess() };
    throttle(process, on, class);
    unsafe {
        SetThreadPriority(GetCurrentThread(), thread);
        if hidden {
            SetProcessWorkingSetSize(process, usize::MAX, usize::MAX);
        }
    }
}

fn throttle(process: HANDLE, on: bool, class: u32) {
    let speed = match on {
        true => PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        false => 0,
    };
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: speed,
        StateMask: speed,
    };
    unsafe {
        SetProcessInformation(
            process,
            ProcessPowerThrottling,
            (&raw const state).cast(),
            size_of_val(&state) as u32,
        );
        SetPriorityClass(process, class);
    }
}
