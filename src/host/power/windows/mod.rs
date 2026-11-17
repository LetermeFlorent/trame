mod family;
#[cfg(test)]
mod tests;
mod tree;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{
    BELOW_NORMAL_PRIORITY_CLASS, GetCurrentProcess, GetCurrentThread, GetProcessInformation,
    IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    ProcessPowerThrottling, SetPriorityClass, SetProcessInformation, SetProcessWorkingSetSize,
    SetThreadPriority, THREAD_PRIORITY_HIGHEST, THREAD_PRIORITY_NORMAL,
};

pub use family::Family;

const LEFT_TO_WINDOWS: PROCESS_POWER_THROTTLING_STATE = PROCESS_POWER_THROTTLING_STATE {
    Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    ControlMask: 0,
    StateMask: 0,
};

pub fn set_own(on: bool, hidden: bool) {
    let (class, thread) = match (on, hidden) {
        (true, true) => (IDLE_PRIORITY_CLASS, THREAD_PRIORITY_HIGHEST),
        (true, false) => (BELOW_NORMAL_PRIORITY_CLASS, THREAD_PRIORITY_NORMAL),
        (false, _) => (NORMAL_PRIORITY_CLASS, THREAD_PRIORITY_NORMAL),
    };
    let process = unsafe { GetCurrentProcess() };
    throttle(process, eco(on), class);
    unsafe {
        SetThreadPriority(GetCurrentThread(), thread);
        if hidden {
            SetProcessWorkingSetSize(process, usize::MAX, usize::MAX);
        }
    }
}

fn eco(on: bool) -> PROCESS_POWER_THROTTLING_STATE {
    PROCESS_POWER_THROTTLING_STATE {
        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: match on {
            true => PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            false => 0,
        },
        ..LEFT_TO_WINDOWS
    }
}

fn speed_of(process: HANDLE) -> Option<PROCESS_POWER_THROTTLING_STATE> {
    let mut speed = LEFT_TO_WINDOWS;
    let read = unsafe {
        GetProcessInformation(
            process,
            ProcessPowerThrottling,
            (&raw mut speed).cast(),
            size_of_val(&speed) as u32,
        )
    };
    (read != 0).then_some(speed)
}

fn throttle(process: HANDLE, speed: PROCESS_POWER_THROTTLING_STATE, class: u32) {
    unsafe {
        SetProcessInformation(
            process,
            ProcessPowerThrottling,
            (&raw const speed).cast(),
            size_of_val(&speed) as u32,
        );
        SetPriorityClass(process, class);
    }
}
