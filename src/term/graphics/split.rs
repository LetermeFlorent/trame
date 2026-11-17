pub fn split(apc: &[u8]) -> Option<(&[u8], &[u8])> {
    let body = apc.strip_prefix(b"G")?;
    Some(match body.iter().position(|&byte| byte == b';') {
        Some(at) => (&body[..at], &body[at + 1..]),
        None => (body, &[]),
    })
}
