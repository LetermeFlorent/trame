use winit::keyboard::{KeyCode, NamedKey};

use super::combo::{Chord, Combo, Trigger};
use crate::ui::Platform;

const CONTROL: Chord = Chord::new(true, false, false, false);
const CONTROL_SHIFT: Chord = Chord::new(true, true, false, false);
const CONTROL_SHIFT_ALT: Chord = Chord::new(true, true, true, false);
const CONTROL_ALT: Chord = Chord::new(true, false, true, false);
const SHIFT: Chord = Chord::new(false, true, false, false);
const ALT: Chord = Chord::new(false, false, true, false);
const LOGO: Chord = Chord::new(false, false, false, true);
const LOGO_SHIFT: Chord = Chord::new(false, true, false, true);
const LOGO_ALT: Chord = Chord::new(false, false, true, true);

struct Layout {
    primary: Chord,
    split_down: Chord,
    select_tab: Chord,
    focus_pane: Chord,
    font: Chord,
    clipboard: bool,
}

impl Layout {
    fn of(platform: Platform) -> Self {
        let mac = platform == Platform::Mac;
        let pick = |on_mac, elsewhere| if mac { on_mac } else { elsewhere };
        Self {
            primary: pick(LOGO, CONTROL_SHIFT),
            split_down: pick(LOGO_SHIFT, CONTROL_SHIFT_ALT),
            select_tab: pick(LOGO, ALT),
            focus_pane: pick(LOGO_ALT, CONTROL_ALT),
            font: pick(LOGO, CONTROL),
            clipboard: platform == Platform::Windows,
        }
    }
}

pub fn default_keys(platform: Platform) -> Vec<Vec<Combo>> {
    let layout = Layout::of(platform);
    let primary = |letter| vec![char(layout.primary, letter)];
    let focus = |edge| vec![named(layout.focus_pane, edge)];
    let mut keys = vec![
        primary('t'),
        primary('w'),
        vec![
            named(CONTROL, NamedKey::Tab),
            named(CONTROL, NamedKey::PageDown),
        ],
        vec![
            named(CONTROL_SHIFT, NamedKey::Tab),
            named(CONTROL, NamedKey::PageUp),
        ],
    ];
    keys.extend((1..=9).map(|digit| vec![digit_key(layout.select_tab, digit)]));
    keys.extend([
        primary('n'),
        vec![named(CONTROL_SHIFT, NamedKey::PageDown)],
        vec![named(CONTROL_SHIFT, NamedKey::PageUp)],
        primary('d'),
        vec![char(layout.split_down, 'd')],
        focus(NamedKey::ArrowLeft),
        focus(NamedKey::ArrowRight),
        focus(NamedKey::ArrowUp),
        focus(NamedKey::ArrowDown),
        primary('b'),
    ]);
    keys.extend(clipboard(&layout));
    keys.extend(font(layout.font));
    keys.extend([
        vec![named(SHIFT, NamedKey::PageUp)],
        vec![named(SHIFT, NamedKey::PageDown)],
        vec![named(CONTROL_SHIFT, NamedKey::Home)],
        vec![named(CONTROL_SHIFT, NamedKey::End)],
    ]);
    keys
}

fn clipboard(layout: &Layout) -> [Vec<Combo>; 2] {
    let mut copy = vec![char(layout.primary, 'c')];
    let mut paste = vec![char(layout.primary, 'v')];
    if layout.clipboard {
        copy.push(char(CONTROL, 'c'));
        paste.push(char(CONTROL, 'v'));
    }
    paste.push(named(SHIFT, NamedKey::Insert));
    [copy, paste]
}

fn font(chord: Chord) -> [Vec<Combo>; 3] {
    [
        vec![char(chord, '='), char(chord, '+')],
        vec![char(chord, '-')],
        vec![
            digit_key(chord, 0),
            Combo::new(chord, Trigger::Numpad(KeyCode::Numpad0)),
        ],
    ]
}

fn char(chord: Chord, single: char) -> Combo {
    Combo::new(chord, Trigger::Char(single))
}

fn digit_key(chord: Chord, digit: u8) -> Combo {
    Combo::new(chord, Trigger::Digit(digit))
}

fn named(chord: Chord, key: NamedKey) -> Combo {
    Combo::new(chord, Trigger::Named(key))
}
