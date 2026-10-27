use super::EditorFrame;
use super::row::{SWATCH_SIZE, SwatchSlot, swatch_row, swatch_slots};
use crate::geometry::Rect;
use crate::locale::Strings;
use crate::preferences::MAX_CUSTOM_COLORS;
use crate::text::TextSystem;
use crate::theme::Swatch;
use crate::ui::frame::editor::{FormAction, FormFrame, button_form};
use crate::ui::frame::sidebar::{CAPTION_HEIGHT, Column, LINE_HEIGHT, ListSpec, SECTION_GAP};
use crate::ui::frame::{FrameLayout, HostList, SettingLine};
use crate::ui::metrics::UiMetrics;

const DOT_GAP: f32 = 8.0;
const ADD_ACTIONS: [FormAction; 1] = [FormAction::Add];

#[derive(Clone, Copy, Debug)]
pub struct ColorsView<'a> {
    pub selected: Option<Swatch>,
    pub custom: &'a [Swatch],
    pub first: usize,
}

impl ColorsView<'_> {
    fn adds(&self) -> bool {
        self.custom.len() < MAX_CUSTOM_COLORS
    }
}

impl FrameLayout {
    pub(in crate::ui::frame::editor) fn color_popup(
        &self,
        text: &mut TextSystem,
        strings: &'static Strings,
        view: ColorsView,
        editor: EditorFrame,
    ) -> EditorFrame {
        let metrics = self.metrics;
        let row = swatch_row(&metrics);
        let tail = custom_chrome(&metrics, view.adds());
        let shown = self.listed_rows(view.custom.len(), row.height + tail);
        let content = row.height + tail + shown as f32 * metrics.px(LINE_HEIGHT);
        let (popup, mut column) = self.open_popup(text, strings.change_color, content);
        let strip = row.take_strip(&mut column);
        let custom = custom_section(&mut column, &view, shown, strings);
        EditorFrame {
            popup: Some(popup),
            slots: swatch_slots(&metrics, strip),
            strip,
            selected: view.selected,
            sections: vec![custom.caption],
            list: custom.list,
            dots: custom.dots,
            form: custom.form,
            ..editor
        }
    }
}

struct CustomSection {
    caption: SettingLine,
    list: Option<HostList>,
    dots: Vec<SwatchSlot>,
    form: Option<FormFrame>,
}

fn custom_section(
    column: &mut Column,
    view: &ColorsView,
    shown: usize,
    strings: &'static Strings,
) -> CustomSection {
    let metrics = column.metrics;
    let caption = column.section(strings.my_colors);
    let labels: Vec<String> = view
        .custom
        .iter()
        .map(|color| String::from(*color))
        .collect();
    let spec = ListSpec {
        labels: &labels,
        first: view.first,
        shown,
        pickable: true,
        forgets: true,
    };
    let mut list = (shown > 0).then(|| column.host_list(&spec));
    let dots = list
        .as_mut()
        .map(|list| custom_dots(&metrics, list, view.custom))
        .unwrap_or_default();
    let form = view.adds().then(|| {
        let top = column.y + metrics.px(SECTION_GAP);
        let height = metrics.px(UiMetrics::BUTTON);
        let area = Rect::new(column.inner.x, top, column.inner.width, height);
        button_form(column.text, &metrics, area, &ADD_ACTIONS, strings)
    });
    CustomSection {
        caption,
        list,
        dots,
        form,
    }
}

fn custom_chrome(metrics: &UiMetrics, adds: bool) -> f32 {
    let caption = metrics.px(SECTION_GAP) + metrics.px(CAPTION_HEIGHT);
    match adds {
        true => caption + metrics.px(SECTION_GAP) + metrics.px(UiMetrics::BUTTON),
        false => caption,
    }
}

fn custom_dots(metrics: &UiMetrics, list: &mut HostList, custom: &[Swatch]) -> Vec<SwatchSlot> {
    let size = metrics.px(SWATCH_SIZE);
    let step = size + metrics.px(DOT_GAP);
    list.rows
        .iter_mut()
        .map(|row| {
            let line = &mut row.line;
            let rect = line.rect.square_at(line.label_x.round(), size);
            line.label_x += step;
            SwatchSlot {
                rect,
                color: custom.get(row.index).copied(),
            }
        })
        .collect()
}
