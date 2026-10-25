use std::time::{Duration, Instant};

const IDLE_AFTER: Duration = Duration::from_secs(30);

pub struct Idle {
    enabled: bool,
    throttled: bool,
    last: Instant,
}

impl Idle {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            throttled: false,
            last: Instant::now(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.touch();
    }

    pub fn touch(&mut self) {
        self.last = Instant::now();
        if self.throttled {
            self.throttled = false;
            set_efficiency(false);
        }
    }

    pub fn deadline(&self) -> Option<Instant> {
        (self.enabled && !self.throttled).then(|| self.last + IDLE_AFTER)
    }

    pub fn tick(&mut self, now: Instant) {
        if self.deadline().is_some_and(|deadline| now >= deadline) {
            self.throttled = true;
            set_efficiency(true);
        }
    }
}

#[cfg(windows)]
fn set_efficiency(on: bool) {
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
        PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        PROCESS_POWER_THROTTLING_STATE, ProcessPowerThrottling, SetPriorityClass,
        SetProcessInformation,
    };
    let throttle = match on {
        true => PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        false => 0,
    };
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: throttle,
        StateMask: throttle,
    };
    let priority = match on {
        true => IDLE_PRIORITY_CLASS,
        false => NORMAL_PRIORITY_CLASS,
    };
    unsafe {
        let process = GetCurrentProcess();
        SetProcessInformation(
            process,
            ProcessPowerThrottling,
            (&raw const state).cast(),
            size_of_val(&state) as u32,
        );
        SetPriorityClass(process, priority);
    }
}

#[cfg(not(windows))]
fn set_efficiency(_on: bool) {}
