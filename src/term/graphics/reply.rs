use super::Error;

const QUIET_OK: u8 = 1;

pub fn reply(id: u32, number: u32, placement: u32, result: Result<(), Error>) -> Vec<u8> {
    let mut keys = format!("i={id}");
    if number != 0 {
        keys.push_str(&format!(",I={number}"));
    }
    if placement != 0 {
        keys.push_str(&format!(",p={placement}"));
    }
    let message = result.map_or_else(|error| error.code(), |()| "OK");
    format!("\x1b_G{keys};{message}\x1b\\").into_bytes()
}

pub fn wanted(quiet: u8, ok: bool) -> bool {
    match quiet {
        0 => true,
        QUIET_OK => !ok,
        _ => false,
    }
}
