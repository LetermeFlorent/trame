const MAX_CHILDREN: usize = 256;
const FIRST_USER_PID: libc::pid_t = 2;

pub fn slow(shell: u32, on: bool) -> bool {
    let Ok(shell) = libc::pid_t::try_from(shell) else {
        return true;
    };
    let mut queue = vec![shell];
    while let Some(pid) = queue.pop() {
        set_background(pid, on);
        queue.extend(children(pid));
    }
    true
}

pub fn set_own(on: bool) {
    set_background(0, on);
}

fn set_background(pid: libc::pid_t, on: bool) {
    let priority = match on {
        true => libc::PRIO_DARWIN_BG,
        false => 0,
    };
    unsafe { libc::setpriority(libc::PRIO_DARWIN_PROCESS, pid as libc::id_t, priority) };
}

fn children(pid: libc::pid_t) -> Vec<libc::pid_t> {
    let mut found: Vec<libc::pid_t> = vec![0; MAX_CHILDREN];
    let size = size_of_val(found.as_slice()) as libc::c_int;
    let count = unsafe { libc::proc_listchildpids(pid, found.as_mut_ptr().cast(), size) };
    found.truncate(usize::try_from(count).unwrap_or(0).min(MAX_CHILDREN));
    found.retain(|&child| child >= FIRST_USER_PID);
    found
}
