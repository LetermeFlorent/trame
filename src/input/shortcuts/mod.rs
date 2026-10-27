mod actions;
mod capture;
mod combo;
mod defaults;
mod fit;
mod keymap;
mod names;
mod overrides;

use winit::keyboard::{Key, ModifiersState, PhysicalKey};

use crate::input::Command;
use crate::ui::Platform;
pub use actions::{ACTIONS, action_of};
pub use capture::{Pressed, pressed};
pub use combo::{Combo, Trigger};
pub use defaults::default_keys;
pub use keymap::Keymap;
pub use overrides::{MAX_KEYS, Overrides};

#[derive(Clone, Copy, Debug)]
pub struct ShortcutInput<'a> {
    pub key: &'a Key,
    pub base: &'a Key,
    pub physical: PhysicalKey,
    pub modifiers: ModifiersState,
}

pub fn shortcut(input: ShortcutInput, platform: Platform) -> Option<Command> {
    Keymap::new(platform, &Overrides::default()).command_for(input)
}
