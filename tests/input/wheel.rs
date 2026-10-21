use trame::input::{WheelAction, arrow_keys, wheel_action};
use trame::term::Modes;

use crate::modes;

#[test]
fn does_nothing_without_movement() {
    assert_eq!(wheel_action(0, Modes::default()), WheelAction::Ignore);
}

#[test]
fn scrolls_the_history_on_the_main_screen() {
    assert_eq!(wheel_action(3, Modes::default()), WheelAction::History(3));
    assert_eq!(wheel_action(-3, Modes::default()), WheelAction::History(-3));
}

#[test]
fn hands_the_wheel_to_programs_that_track_the_mouse() {
    assert_eq!(wheel_action(1, modes(&[Modes::MOUSE_CLICK])), WheelAction::Report);
    let full_screen = modes(&[Modes::ALT_SCREEN, Modes::MOUSE_MOTION]);
    assert_eq!(wheel_action(-1, full_screen), WheelAction::Report);
}

#[test]
fn turns_the_wheel_into_arrows_on_the_alternate_screen() {
    let full_screen = modes(&[Modes::ALT_SCREEN]);
    assert_eq!(wheel_action(3, full_screen), WheelAction::Keys(3, true));
    assert_eq!(wheel_action(-2, full_screen), WheelAction::Keys(2, false));
}

#[test]
fn ignores_the_wheel_when_alternate_scroll_is_off() {
    let mut full_screen = modes(&[Modes::ALT_SCREEN]);
    full_screen.set(Modes::ALTERNATE_SCROLL, false);
    assert_eq!(wheel_action(3, full_screen), WheelAction::Ignore);
}

#[test]
fn repeats_arrow_keys_in_the_cursor_mode_of_the_program() {
    assert_eq!(arrow_keys(2, true, Modes::default()), b"\x1b[A\x1b[A");
    assert_eq!(arrow_keys(1, false, modes(&[Modes::APP_CURSOR])), b"\x1bOB");
    assert!(arrow_keys(0, true, Modes::default()).is_empty());
}
