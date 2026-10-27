mod family;
mod tree;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    PROCESS_POWER_THROTTLING_STATE, ProcessPowerThrottling, SetPriorityClass,
    SetProcessInformation,
};

pub use family::Family;

pub fn set_own(on: bool) {
    let class = match on {
        true => IDLE_PRIORITY_CLASS,
        false => NORMAL_PRIORITY_CLASS,
    };
    throttle(unsafe { GetCurrentProcess() }, on, class);
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
