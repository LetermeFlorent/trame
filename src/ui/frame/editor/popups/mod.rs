mod colors;
mod connect;
mod emblems;
mod picker;
mod row;

use super::EditorFrame;
use crate::geometry::Rect;
use crate::locale::Strings;
use crate::text::TextSystem;
use crate::theme::Emblem;
use crate::ui::frame::sidebar::LINE_HEIGHT;
use crate::ui::frame::{FrameLayout, PopupFrame};
pub use colors::ColorsView;
pub(super) use connect::ConnectDialog;
pub use emblems::EmblemSlot;
use emblems::{emblem_row, emblem_slots};
pub use picker::{PickerFrame, PickerPart, PickerView};
use row::RowSize;
pub use row::SwatchSlot;

const MAX_LISTED: usize = 6;

impl FrameLayout {
    pub(super) fn emblem_popup(
        &self,
        text: &mut TextSystem,
        strings: &Strings,
        emblem: Option<Emblem>,
        editor: EditorFrame,
    ) -> EditorFrame {
        let (popup, strip) = self.row_popup(text, strings.change_icon, emblem_row(&self.metrics));
        EditorFrame {
            popup: Some(popup),
            emblems: emblem_slots(&self.metrics, strip),
            strip,
            emblem,
            ..editor
        }
    }

    fn row_popup(&self, text: &mut TextSystem, title: &str, row: RowSize) -> (PopupFrame, Rect) {
        let (popup, column) = self.open_popup(text, title, row.height);
        let x = column.inner.centered_x(row.width).round();
        (popup, Rect::new(x, column.y, row.width, row.height))
    }

    fn listed_rows(&self, count: usize, other: f32) -> usize {
        if count == 0 {
            return 0;
        }
        let room = self.popup_room() - other;
        let fits = (room / self.metrics.px(LINE_HEIGHT)).floor().max(0.0) as usize;
        fits.min(count).min(MAX_LISTED)
    }
}
