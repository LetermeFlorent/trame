use winit::window::CursorIcon;

use trame::ui::Hit;
use trame::ui::frame::PickerPart;
use trame::workspace::{Axis, Divider};

use crate::app::window;

pub(super) fn hover_cursor_for(
    hover: Option<Hit>,
    dividers: &[Divider],
    clickable: bool,
) -> CursorIcon {
    if clickable {
        CursorIcon::Pointer
    } else {
        cursor_for(hover, dividers)
    }
}

fn cursor_for(hit: Option<Hit>, dividers: &[Divider]) -> CursorIcon {
    match hit {
        Some(Hit::Resize(edge)) => window::resize_cursor(edge),
        Some(Hit::Divider(index)) => dividers
            .get(index)
            .map_or(CursorIcon::Default, |divider| divider_cursor(divider.axis)),
        Some(Hit::SidebarEdge) => CursorIcon::ColResize,
        Some(
            Hit::Terminal(_) | Hit::EditField | Hit::FormField(_) | Hit::Picker(PickerPart::Field),
        ) => CursorIcon::Text,
        None
        | Some(
            Hit::TitleBar
            | Hit::Sidebar
            | Hit::TabBar(_)
            | Hit::EditStrip
            | Hit::SettingsPanel
            | Hit::Menu
            | Hit::Dialog
            | Hit::Backdrop,
        ) => CursorIcon::Default,
        Some(_) => CursorIcon::Pointer,
    }
}

pub(in crate::app::running) fn divider_cursor(axis: Axis) -> CursorIcon {
    match axis {
        Axis::Row => CursorIcon::ColResize,
        Axis::Column => CursorIcon::RowResize,
    }
}

#[cfg(test)]
mod tests {
    use trame::workspace::PaneId;

    use super::*;

    #[test]
    fn hover_cursor_switches_to_pointer_over_clickable_picture() {
        let hit = Some(Hit::Terminal(PaneId(1)));
        assert_eq!(hover_cursor_for(hit, &[], false), CursorIcon::Text);
        assert_eq!(hover_cursor_for(hit, &[], true), CursorIcon::Pointer);
    }
}
