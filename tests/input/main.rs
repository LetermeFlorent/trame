mod field;
mod keys;
mod mouse;
mod paste;
mod shortcuts;
mod wheel;

use trame::term::Modes;
use winit::keyboard::Key;

pub fn modes(enabled: &[Modes]) -> Modes {
    let mut modes = Modes::default();
    for flag in enabled {
        modes.set(*flag, true);
    }
    modes
}

pub fn character(text: &str) -> Key {
    Key::Character(text.into())
}
