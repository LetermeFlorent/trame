use std::fs::OpenOptions;
use std::io::Write;

const OWN_TIMER_SLACK_NS: libc::c_ulong = 50_000_000;

pub fn slow(shell: u32, on: bool) -> bool {
    let nice: &[u8] = match on {
        true => b"19",
        false => b"0",
    };
    let path = format!("/proc/{shell}/autogroup");
    let written = OpenOptions::new()
        .write(true)
        .open(path)
        .and_then(|mut file| file.write_all(nice));
    !matches!(written, Err(error) if error.raw_os_error() == Some(libc::EAGAIN))
}

pub fn set_own(on: bool) {
    let (slack, policy) = match on {
        true => (OWN_TIMER_SLACK_NS, libc::SCHED_BATCH),
        false => (0, libc::SCHED_OTHER),
    };
    let param = libc::sched_param { sched_priority: 0 };
    unsafe {
        libc::prctl(libc::PR_SET_TIMERSLACK, slack);
        libc::sched_setscheduler(0, policy, &param);
    }
}
