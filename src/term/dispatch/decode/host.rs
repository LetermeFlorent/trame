use std::sync::OnceLock;

const LOCALHOST: &[u8] = b"localhost";
#[cfg(unix)]
const MAX_HOST_NAME: usize = 256;

pub(super) fn is_this_machine(host: &[u8]) -> bool {
    let label = first_label(host);
    host.is_empty()
        || label.eq_ignore_ascii_case(LOCALHOST)
        || machine_name().is_some_and(|name| label.eq_ignore_ascii_case(first_label(name)))
}

fn first_label(host: &[u8]) -> &[u8] {
    host.split(|&byte| byte == b'.').next().unwrap_or_default()
}

fn machine_name() -> Option<&'static [u8]> {
    static NAME: OnceLock<Option<String>> = OnceLock::new();
    let name = NAME.get_or_init(|| read_machine_name().filter(|name| !name.is_empty()));
    name.as_deref().map(str::as_bytes)
}

#[cfg(windows)]
fn read_machine_name() -> Option<String> {
    std::env::var("COMPUTERNAME").ok()
}

#[cfg(unix)]
fn read_machine_name() -> Option<String> {
    let mut buffer = [0u8; MAX_HOST_NAME];
    let read = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) };
    let end = buffer.iter().position(|&byte| byte == 0)?;
    let name = std::str::from_utf8(&buffer[..end]).ok()?;
    (read == 0).then(|| name.to_owned())
}
