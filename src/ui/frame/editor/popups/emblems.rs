use super::EditorFrame;
use super::row::{RowSize, slot_rects};
use crate::geometry::Rect;
use crate::input::TextField;
use crate::locale::Strings;
use crate::text::TextSystem;
use crate::theme::Emblem;
use crate::ui::frame::FrameLayout;
use crate::ui::frame::editor::{FormAction, button_form, place_field};
use crate::ui::frame::sidebar::{CAPTION_HEIGHT, SECTION_GAP};
use crate::ui::metrics::UiMetrics;

const EMBLEM_SLOT: f32 = 18.0;
const EMBLEM_GAP: f32 = 4.0;
const LETTERS_WIDTH: f32 = 64.0;
const CUSTOM_ACTIONS: [FormAction; 2] = [FormAction::Image, FormAction::Apply];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmblemSlot {
    pub rect: Rect,
    pub icon: Rect,
    pub emblem: Emblem,
}

#[derive(Clone, Copy, Debug)]
pub struct EmblemView<'a> {
    pub selected: Option<Emblem>,
    pub letters: &'a TextField,
}

impl FrameLayout {
    pub(in crate::ui::frame::editor) fn emblem_popup(
        &self,
        text: &mut TextSystem,
        strings: &'static Strings,
        view: EmblemView,
        editor: EditorFrame,
    ) -> EditorFrame {
        let metrics = self.metrics;
        let row = emblem_row(&metrics);
        let line = metrics.px(UiMetrics::SIDEBAR_ROW);
        let caption = metrics.px(SECTION_GAP) + metrics.px(CAPTION_HEIGHT);
        let (popup, mut column) =
            self.open_popup(text, strings.change_icon, row.height + caption + line);
        let x = column.inner.centered_x(row.width).round();
        let strip = Rect::new(x, column.y, row.width, row.height);
        column.y = strip.bottom();
        let caption = column.section(strings.letters_or_image);
        let inner = column.inner;
        let slot = Rect::new(inner.x, column.y, metrics.px(LETTERS_WIDTH), line);
        let field = place_field(column.text, &metrics, slot, inner, view.letters);
        let side = metrics.px(UiMetrics::BUTTON);
        let area = Rect::new(inner.x, slot.centered_y(side).round(), inner.width, side);
        let form = button_form(column.text, &metrics, area, &CUSTOM_ACTIONS, strings);
        EditorFrame {
            popup: Some(popup),
            emblems: emblem_slots(&metrics, strip),
            strip,
            emblem: view.selected,
            field: Some(field),
            sections: vec![caption],
            form: Some(form),
            ..editor
        }
    }
}

fn emblem_row(metrics: &UiMetrics) -> RowSize {
    RowSize::of(metrics, Emblem::ALL.len(), EMBLEM_SLOT, EMBLEM_GAP)
}

fn emblem_slots(metrics: &UiMetrics, strip: Rect) -> Vec<EmblemSlot> {
    let size = metrics.px(EMBLEM_SLOT);
    let inset = (size - metrics.px(UiMetrics::ICON)) / 2.0;
    slot_rects(metrics, strip, Emblem::ALL.len(), EMBLEM_SLOT, EMBLEM_GAP)
        .zip(Emblem::ALL)
        .map(|(rect, emblem)| EmblemSlot {
            rect,
            icon: rect.inset(inset),
            emblem,
        })
        .collect()
}
