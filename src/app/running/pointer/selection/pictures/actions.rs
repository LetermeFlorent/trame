use winit::event::{ElementState, MouseButton as WinitButton};
use winit::window::CursorIcon;

use trame::geometry::{Point, Rect};
use trame::input::MouseButton;
use trame::ui::chrome::PictureButton;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app::running::pointer::selection) enum PictureAction {
    Open,
    Close,
    Pass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PictureKeyAction {
    Pass,
    Consume,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PictureButtonHit {
    WindowControl,
    SidebarToggle,
    OpenButton,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PictureButtonAction {
    Pass,
    Consume,
    Close,
    OpenExternal,
}

pub(in crate::app::running::pointer::selection) fn picture_action(
    open: bool,
    button: MouseButton,
    reports: bool,
    primary: bool,
) -> PictureAction {
    match (open, button) {
        (true, MouseButton::Left) => PictureAction::Close,
        (true, _) => PictureAction::Pass,
        (false, MouseButton::Left) if !reports || primary => PictureAction::Open,
        (false, _) => PictureAction::Pass,
    }
}

pub(super) fn picture_under_allowed(selecting: bool, reports: bool, primary: bool) -> bool {
    !selecting && picture_action(false, MouseButton::Left, reports, primary) == PictureAction::Open
}

pub(super) fn picture_key_action(open: bool, blocked: bool, escape: bool) -> PictureKeyAction {
    match (open && !blocked, escape) {
        (true, true) => PictureKeyAction::Close,
        (true, false) => PictureKeyAction::Consume,
        (false, _) => PictureKeyAction::Pass,
    }
}

pub(super) fn picture_button_action(
    open: bool,
    blocked: bool,
    hit: PictureButtonHit,
    state: ElementState,
    button: WinitButton,
) -> PictureButtonAction {
    if !open {
        return PictureButtonAction::Pass;
    }
    if blocked {
        return match hit {
            PictureButtonHit::WindowControl => PictureButtonAction::Pass,
            _ => PictureButtonAction::Consume,
        };
    }
    match (state, button, hit) {
        (ElementState::Pressed, WinitButton::Left, PictureButtonHit::OpenButton) => {
            PictureButtonAction::OpenExternal
        }
        (ElementState::Pressed, WinitButton::Left, _) => PictureButtonAction::Close,
        _ => PictureButtonAction::Consume,
    }
}

pub(super) fn picture_hover_hit(
    close: Rect,
    open: Option<Rect>,
    position: Option<Point>,
) -> Option<PictureButton> {
    let point = position?;
    if close.contains(point) {
        Some(PictureButton::Close)
    } else if open.is_some_and(|rect| rect.contains(point)) {
        Some(PictureButton::Open)
    } else {
        None
    }
}

pub(super) fn picture_cursor(
    close: Rect,
    open: Option<Rect>,
    position: Option<Point>,
) -> CursorIcon {
    match picture_hover_hit(close, open, position) {
        Some(_) => CursorIcon::Pointer,
        None => CursorIcon::Default,
    }
}
