use std::path::PathBuf;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub(super) fn process_cwd(pid: libc::pid_t) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{pid}/cwd")).ok()
}

#[cfg(target_os = "macos")]
pub(super) fn process_cwd(pid: libc::pid_t) -> Option<PathBuf> {
    use std::ffi::OsStr;
    use std::mem::{size_of, size_of_val, zeroed};
    use std::os::unix::ffi::OsStrExt;

    let mut info: libc::proc_vnodepathinfo = unsafe { zeroed() };
    let size = size_of::<libc::proc_vnodepathinfo>() as libc::c_int;
    let read = unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDVNODEPATHINFO,
            0,
            (&mut info as *mut libc::proc_vnodepathinfo).cast(),
            size,
        )
    };
    if read != size {
        return None;
    }
    let path = &info.pvi_cdir.vip_path;
    let bytes: &[u8] =
        unsafe { std::slice::from_raw_parts(path.as_ptr().cast(), size_of_val(path)) };
    let end = bytes.iter().position(|&byte| byte == 0)?;
    Some(PathBuf::from(OsStr::from_bytes(&bytes[..end])))
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos")))]
pub(super) fn process_cwd(_pid: libc::pid_t) -> Option<PathBuf> {
    None
}
