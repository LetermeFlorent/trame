use std::sync::Arc;

use crate::geometry::Rect;
use crate::locale::Strings;
use crate::text::{TextLayout, TextSystem};
use crate::ui::metrics::UiMetrics;

const BUTTON_PAD_X: f32 = 12.0;
const BUTTON_GAP: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormAction {
    Connect,
    Save,
    Cancel,
    Close,
    Change,
    Apply,
    Add,
    Image,
    Restart,
}

impl FormAction {
    pub fn is_primary(self) -> bool {
        !matches!(self, Self::Cancel | Self::Change | Self::Add | Self::Image)
    }

    fn label(self, strings: &'static Strings) -> &'static str {
        match self {
            Self::Connect => strings.connect,
            Self::Save => strings.save,
            Self::Cancel => strings.cancel,
            Self::Close => strings.close,
            Self::Change => strings.change,
            Self::Apply => strings.apply,
            Self::Add => strings.add,
            Self::Image => strings.image,
            Self::Restart => strings.restart,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FormButton {
    pub rect: Rect,
    pub label: Arc<TextLayout>,
    pub action: FormAction,
}

pub(in crate::ui::frame) fn place_buttons(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    area: Rect,
    actions: &[FormAction],
    strings: &'static Strings,
) -> Vec<FormButton> {
    let height = metrics.px(UiMetrics::BUTTON);
    let gap = metrics.px(BUTTON_GAP);
    let mut right = area.right();
    let mut buttons: Vec<FormButton> = actions
        .iter()
        .rev()
        .map(|&action| {
            let label = text.layout(action.label(strings), metrics.label());
            let width = label.width.ceil() + 2.0 * metrics.px(BUTTON_PAD_X);
            let rect = Rect::new(right - width, area.bottom() - height, width, height);
            right = rect.x - gap;
            FormButton {
                rect,
                label,
                action,
            }
        })
        .collect();
    buttons.reverse();
    buttons
}
