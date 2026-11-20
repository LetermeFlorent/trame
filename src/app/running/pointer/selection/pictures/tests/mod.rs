mod hover;
mod state;

use trame::geometry::{Point, Rect};
use trame::input::MouseButton::{Left, Middle, Right};
use winit::event::{ElementState, MouseButton as WinitButton};
use winit::window::CursorIcon;

use super::actions::PictureAction::*;
use super::actions::{
    PictureButtonAction, PictureButtonHit, PictureKeyAction, picture_action, picture_button_action,
    picture_cursor, picture_key_action, picture_under_allowed,
};
use crate::app::running::pointer::scroll::wheel_blocked;

#[test]
fn picture_action_handles_clicks_and_mouse_reporting() {
    for (case, want) in [
        ((false, Left, false, false), Open),
        ((false, Left, true, false), Pass),
        ((false, Left, true, true), Open),
        ((false, Right, false, true), Pass),
        ((true, Left, false, false), Close),
        ((true, Middle, true, true), Pass),
    ] {
        assert_eq!(picture_action(case.0, case.1, case.2, case.3), want);
    }
}

#[test]
fn picture_hover_blocked_while_selecting_or_held() {
    assert!(picture_under_allowed(false, false, false));
    assert!(!picture_under_allowed(true, false, false));
    assert!(!picture_under_allowed(true, true, true));
}

#[test]
fn picture_key_closes_on_escape_and_consumes_when_unblocked() {
    assert_eq!(
        picture_key_action(true, false, true),
        PictureKeyAction::Close
    );
    assert_eq!(
        picture_key_action(true, false, false),
        PictureKeyAction::Consume
    );
    assert_eq!(picture_key_action(true, true, true), PictureKeyAction::Pass);
    assert_eq!(
        picture_key_action(true, true, false),
        PictureKeyAction::Pass
    );
    assert_eq!(
        picture_key_action(false, false, true),
        PictureKeyAction::Pass
    );
}

#[test]
fn picture_button_handles_close_open_and_sidebar_toggle() {
    use {ElementState as S, PictureButtonAction as Act, PictureButtonHit as H, WinitButton as B};
    let cases = [
        (false, false, H::Other, S::Pressed, B::Left, Act::Pass),
        (true, false, H::Other, S::Pressed, B::Left, Act::Close),
        (
            true,
            false,
            H::OpenButton,
            S::Pressed,
            B::Left,
            Act::OpenExternal,
        ),
        (true, false, H::Other, S::Pressed, B::Right, Act::Consume),
        (true, false, H::Other, S::Released, B::Middle, Act::Consume),
        (true, true, H::WindowControl, S::Pressed, B::Left, Act::Pass),
        (
            true,
            true,
            H::SidebarToggle,
            S::Pressed,
            B::Left,
            Act::Consume,
        ),
    ];
    for (open, under, hit, state, btn, want) in cases {
        assert_eq!(picture_button_action(open, under, hit, state, btn), want);
    }
}

#[test]
fn picture_cursor_is_pointer_on_close_or_open_button_and_default_over_scrim() {
    let close = Rect::new(900.0, 40.0, 28.0, 28.0);
    let open = Some(Rect::new(760.0, 40.0, 132.0, 28.0));
    assert_eq!(
        picture_cursor(close, open, Some(Point::new(910.0, 50.0))),
        CursorIcon::Pointer
    );
    assert_eq!(
        picture_cursor(close, open, Some(Point::new(800.0, 50.0))),
        CursorIcon::Pointer
    );
    assert_eq!(
        picture_cursor(close, open, Some(Point::new(100.0, 100.0))),
        CursorIcon::Default
    );
    assert_eq!(picture_cursor(close, open, None), CursorIcon::Default);
}

#[test]
fn picture_wheel_returns_early_while_picture_is_open() {
    assert!(wheel_blocked(true, false, false, true));
    assert!(wheel_blocked(true, false, false, false));
    assert!(!wheel_blocked(false, false, false, false));
}

#[test]
fn blocked_picture_consumes_non_closing_clicks() {
    use {ElementState as S, PictureButtonAction as Act, PictureButtonHit as H, WinitButton as B};
    for hit in [H::Other, H::OpenButton, H::SidebarToggle] {
        assert_eq!(
            picture_button_action(true, true, hit, S::Pressed, B::Left),
            Act::Consume
        );
    }
    assert_eq!(
        picture_button_action(true, true, H::WindowControl, S::Pressed, B::Left),
        Act::Pass
    );
}
