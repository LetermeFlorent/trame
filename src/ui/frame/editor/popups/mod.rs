mod colors;
mod connect;
mod emblems;
mod picker;
mod row;

use super::EditorFrame;
use crate::ui::frame::FrameLayout;
use crate::ui::frame::sidebar::LINE_HEIGHT;
pub use colors::ColorsView;
pub(super) use connect::ConnectDialog;
pub use emblems::{EmblemSlot, EmblemView};
pub use picker::{PickerFrame, PickerPart, PickerView};
pub use row::SwatchSlot;

const MAX_LISTED: usize = 6;

impl FrameLayout {
    fn listed_rows(&self, count: usize, other: f32) -> usize {
        if count == 0 {
            return 0;
        }
        let room = self.popup_room() - other;
        let fits = (room / self.metrics.px(LINE_HEIGHT)).floor().max(0.0) as usize;
        fits.min(count).min(MAX_LISTED)
    }
}
