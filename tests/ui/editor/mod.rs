mod custom;
mod emblems;
mod form;
mod menu;
mod picker;
mod rename;
mod servers;
mod sidebar;
mod swatches;

use trame::geometry::Point;
use trame::locale::Language;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::Swatch;
use trame::ui::frame::{ColorsView, EditAnchor, EditKind, EditTarget, EditorRequest, PopupFrame};
use trame::ui::{FrameLayout, Hit};
use trame::workspace::Workspace;

use crate::{frame, text_system, workspace};

fn place(
    text: &mut TextSystem,
    workspace: &Workspace,
    target: EditTarget,
    anchor: EditAnchor,
    kind: EditKind,
) -> FrameLayout {
    let mut frame = frame(text, workspace);
    frame.place_editor(
        text,
        &EditorRequest {
            target,
            anchor,
            kind,
            strings: Language::English.strings(),
        },
    );
    frame
}

fn on_strip(text: &mut TextSystem, workspace: &Workspace, kind: EditKind) -> FrameLayout {
    let strip = EditAnchor::Strip(workspace.instances[0].active_pane);
    place(text, workspace, EditTarget::Tab(SessionId(2)), strip, kind)
}

fn colors(selected: Option<Swatch>) -> EditKind<'static> {
    EditKind::Color(ColorsView {
        selected,
        custom: &[],
        first: 0,
        filled: false,
    })
}

#[test]
fn no_editor_is_placed_until_one_is_requested() {
    let mut text = text_system();
    assert!(frame(&mut text, &workspace()).editor.is_none());
}

#[track_caller]
fn closing_popup(frame: &FrameLayout) -> &PopupFrame {
    let popup = frame
        .editor
        .as_ref()
        .and_then(|editor| editor.popup.as_ref());
    let popup = popup.expect("a popup");
    assert_eq!(frame.hit(popup.close.center(), false), Some(Hit::EditClose));
    let outside = Point::new(popup.rect.x - 10.0, popup.rect.center().y);
    assert_eq!(frame.hit(outside, false), Some(Hit::Backdrop));
    popup
}
