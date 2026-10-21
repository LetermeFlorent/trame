use winit::keyboard::KeyCode;

use crate::input::Command;
use crate::ui::Platform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Chord {
    pub control: bool,
    pub shift: bool,
    pub alt: bool,
    pub logo: bool,
}

const fn chord(control: bool, shift: bool, alt: bool, logo: bool) -> Chord {
    Chord {
        control,
        shift,
        alt,
        logo,
    }
}

pub(super) const CONTROL: Chord = chord(true, false, false, false);
pub(super) const CONTROL_SHIFT: Chord = chord(true, true, false, false);
pub(super) const CONTROL_SHIFT_ALT: Chord = chord(true, true, true, false);
pub(super) const CONTROL_ALT: Chord = chord(true, false, true, false);
pub(super) const SHIFT: Chord = chord(false, true, false, false);
pub(super) const ALT: Chord = chord(false, false, true, false);
pub(super) const LOGO: Chord = chord(false, false, false, true);
pub(super) const LOGO_SHIFT: Chord = chord(false, true, false, true);
pub(super) const LOGO_ALT: Chord = chord(false, false, true, true);

pub(super) const LETTERS: [(&str, Command); 7] = [
    ("t", Command::NewTab),
    ("w", Command::CloseTab),
    ("n", Command::NewInstance),
    ("d", Command::SplitRight),
    ("b", Command::ToggleSidebar),
    ("c", Command::Copy),
    ("v", Command::Paste),
];

pub(super) const DIGITS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

pub(super) struct Bindings {
    pub primary: Chord,
    pub split_down: Chord,
    pub select_tab: Chord,
    pub focus_pane: Chord,
    pub font: Chord,
}

impl Bindings {
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::Mac => Self {
                primary: LOGO,
                split_down: LOGO_SHIFT,
                select_tab: LOGO,
                focus_pane: LOGO_ALT,
                font: LOGO,
            },
            Platform::Windows | Platform::Linux => Self {
                primary: CONTROL_SHIFT,
                split_down: CONTROL_SHIFT_ALT,
                select_tab: ALT,
                focus_pane: CONTROL_ALT,
                font: CONTROL,
            },
        }
    }
}
