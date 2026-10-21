use crate::term::Modes;

const LEGACY_PREFIX: &[u8] = b"\x1b[M";
const LEGACY_MAX_COORDINATE: usize = 223;
const LEGACY_OFFSET: u8 = 32;
const LEGACY_RELEASE: u8 = 3;
const NO_BUTTON: u8 = 3;
const MOTION_BIT: u8 = 32;
const SHIFT_BIT: u8 = 4;
const ALT_BIT: u8 = 8;
const CTRL_BIT: u8 = 16;
const WHEEL_UP_CODE: u8 = 64;
const WHEEL_DOWN_CODE: u8 = 65;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseAction {
    Press,
    Release,
    Motion,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MouseModifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    pub action: MouseAction,
    pub button: Option<MouseButton>,
    pub col: usize,
    pub row: usize,
    pub modifiers: MouseModifiers,
}

pub fn encode_mouse(event: &MouseEvent, modes: Modes) -> Option<Vec<u8>> {
    if !is_wanted(event, modes) {
        return None;
    }
    let sgr = modes.contains(Modes::MOUSE_SGR);
    let code = button_code(event, sgr)? | modifier_bits(event.modifiers);
    if sgr {
        let last = if event.action == MouseAction::Release {
            'm'
        } else {
            'M'
        };
        let report = format!("\x1b[<{code};{};{}{last}", event.col + 1, event.row + 1);
        return Some(report.into_bytes());
    }
    let mut bytes = LEGACY_PREFIX.to_vec();
    bytes.push(LEGACY_OFFSET + code);
    bytes.push(legacy_coordinate(event.col));
    bytes.push(legacy_coordinate(event.row));
    Some(bytes)
}

fn is_wanted(event: &MouseEvent, modes: Modes) -> bool {
    match (event.action, event.button) {
        (MouseAction::Motion, None) => modes.contains(Modes::MOUSE_MOTION),
        (MouseAction::Motion, Some(_)) => {
            modes.intersects(Modes::MOUSE_DRAG) || modes.contains(Modes::MOUSE_MOTION)
        }
        _ => modes.intersects(Modes::MOUSE_ANY),
    }
}

fn button_code(event: &MouseEvent, sgr: bool) -> Option<u8> {
    let held = event.button.map(base_code);
    match event.action {
        MouseAction::Press => held,
        MouseAction::Release => match event.button? {
            MouseButton::WheelUp | MouseButton::WheelDown => None,
            _ if sgr => held,
            _ => Some(LEGACY_RELEASE),
        },
        MouseAction::Motion => Some(held.unwrap_or(NO_BUTTON) | MOTION_BIT),
    }
}

fn base_code(button: MouseButton) -> u8 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        MouseButton::WheelUp => WHEEL_UP_CODE,
        MouseButton::WheelDown => WHEEL_DOWN_CODE,
    }
}

fn modifier_bits(modifiers: MouseModifiers) -> u8 {
    let bit = |held: bool, value: u8| if held { value } else { 0 };
    bit(modifiers.shift, SHIFT_BIT) | bit(modifiers.alt, ALT_BIT) | bit(modifiers.ctrl, CTRL_BIT)
}

fn legacy_coordinate(zero_based: usize) -> u8 {
    let one_based = (zero_based + 1).min(LEGACY_MAX_COORDINATE);
    LEGACY_OFFSET + one_based as u8
}
