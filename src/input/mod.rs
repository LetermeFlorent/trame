mod command;
mod keys;
mod mouse;
mod paste;
mod shortcuts;
mod wheel;

pub use command::Command;
pub use keys::{KeyInput, encode};
pub use mouse::{MouseAction, MouseButton, MouseEvent, MouseModifiers, encode_mouse};
pub use paste::paste_bytes;
pub use shortcuts::{ShortcutInput, shortcut};
pub use wheel::{WheelAction, arrow_keys, wheel_action};
