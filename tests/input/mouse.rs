use trame::input::{MouseAction, MouseButton, MouseEvent, MouseModifiers, encode_mouse};
use trame::term::Modes;

use crate::modes;

const ESC_M: [u8; 3] = [0x1b, b'[', b'M'];

fn event(action: MouseAction, button: Option<MouseButton>, col: usize, row: usize) -> MouseEvent {
    MouseEvent {
        action,
        button,
        col,
        row,
        modifiers: MouseModifiers::default(),
    }
}

fn press(button: MouseButton, col: usize, row: usize) -> MouseEvent {
    event(MouseAction::Press, Some(button), col, row)
}

fn release(button: MouseButton, col: usize, row: usize) -> MouseEvent {
    event(MouseAction::Release, Some(button), col, row)
}

fn motion(held: Option<MouseButton>, col: usize, row: usize) -> MouseEvent {
    event(MouseAction::Motion, held, col, row)
}

fn sgr_click() -> Modes {
    modes(&[Modes::MOUSE_CLICK, Modes::MOUSE_SGR])
}

fn legacy(code: u8, col: u8, row: u8) -> Option<Vec<u8>> {
    Some([&ESC_M[..], &[code, col, row]].concat())
}

#[test]
fn reports_nothing_without_a_mouse_mode() {
    let click = press(MouseButton::Left, 0, 0);
    assert_eq!(encode_mouse(&click, Modes::default()), None);
    assert_eq!(encode_mouse(&click, modes(&[Modes::MOUSE_SGR])), None);
}

#[test]
fn encodes_sgr_press_and_release_with_one_based_cells() {
    let found = encode_mouse(&press(MouseButton::Left, 4, 9), sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<0;5;10M"[..]));
    let found = encode_mouse(&release(MouseButton::Left, 4, 9), sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<0;5;10m"[..]));
    let found = encode_mouse(&press(MouseButton::Right, 0, 0), sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<2;1;1M"[..]));
    let found = encode_mouse(&press(MouseButton::Middle, 0, 0), sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<1;1;1M"[..]));
}

#[test]
fn sgr_release_keeps_the_button() {
    let found = encode_mouse(&release(MouseButton::Right, 0, 0), sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<2;1;1m"[..]));
}

#[test]
fn sgr_does_not_cap_large_coordinates() {
    let found = encode_mouse(&press(MouseButton::Left, 499, 299), sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<0;500;300M"[..]));
}

#[test]
fn encodes_legacy_with_offset_bytes() {
    let click = modes(&[Modes::MOUSE_CLICK]);
    let found = encode_mouse(&press(MouseButton::Left, 0, 0), click);
    assert_eq!(found, legacy(32, 33, 33));
    let found = encode_mouse(&press(MouseButton::Right, 9, 4), click);
    assert_eq!(found, legacy(34, 42, 37));
}

#[test]
fn legacy_release_loses_the_button() {
    let click = modes(&[Modes::MOUSE_CLICK]);
    let found = encode_mouse(&release(MouseButton::Right, 0, 0), click);
    assert_eq!(found, legacy(35, 33, 33));
}

#[test]
fn legacy_coordinates_are_capped_at_223() {
    let click = modes(&[Modes::MOUSE_CLICK]);
    let found = encode_mouse(&press(MouseButton::Left, 222, 500), click);
    assert_eq!(found, legacy(32, 255, 255));
    let found = encode_mouse(&press(MouseButton::Left, 223, 1000), click);
    assert_eq!(found, legacy(32, 255, 255));
}

#[test]
fn modifiers_add_their_bits() {
    let mut all = press(MouseButton::Left, 0, 0);
    all.modifiers = MouseModifiers {
        shift: true,
        alt: true,
        ctrl: true,
    };
    let found = encode_mouse(&all, sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<28;1;1M"[..]));
    let mut ctrl = press(MouseButton::Middle, 0, 0);
    ctrl.modifiers.ctrl = true;
    let found = encode_mouse(&ctrl, sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<17;1;1M"[..]));
}

#[test]
fn wheel_is_reported_as_press_only() {
    let found = encode_mouse(&press(MouseButton::WheelUp, 2, 3), sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<64;3;4M"[..]));
    let found = encode_mouse(&press(MouseButton::WheelDown, 2, 3), sgr_click());
    assert_eq!(found.as_deref(), Some(&b"\x1b[<65;3;4M"[..]));
    assert_eq!(
        encode_mouse(&release(MouseButton::WheelUp, 2, 3), sgr_click()),
        None
    );
}

#[test]
fn click_mode_ignores_motion() {
    let dragged = motion(Some(MouseButton::Left), 1, 1);
    assert_eq!(encode_mouse(&dragged, sgr_click()), None);
    assert_eq!(encode_mouse(&motion(None, 1, 1), sgr_click()), None);
}

#[test]
fn drag_mode_reports_motion_only_with_a_button() {
    let drag = modes(&[Modes::MOUSE_DRAG, Modes::MOUSE_SGR]);
    let found = encode_mouse(&motion(Some(MouseButton::Left), 1, 2), drag);
    assert_eq!(found.as_deref(), Some(&b"\x1b[<32;2;3M"[..]));
    assert_eq!(encode_mouse(&motion(None, 1, 2), drag), None);
}

#[test]
fn motion_mode_reports_hover_as_button_three() {
    let all = modes(&[Modes::MOUSE_MOTION, Modes::MOUSE_SGR]);
    let found = encode_mouse(&motion(None, 1, 2), all);
    assert_eq!(found.as_deref(), Some(&b"\x1b[<35;2;3M"[..]));
    let found = encode_mouse(&motion(Some(MouseButton::Right), 1, 2), all);
    assert_eq!(found.as_deref(), Some(&b"\x1b[<34;2;3M"[..]));
    let found = encode_mouse(&motion(None, 0, 0), modes(&[Modes::MOUSE_MOTION]));
    assert_eq!(found, legacy(67, 33, 33));
}
