use super::EditorFrame;
use crate::geometry::Rect;
use crate::locale::Strings;
use crate::text::TextSystem;
use crate::ui::frame::editor::{FormAction, FormSpec, form_height, form_title, place_form};
use crate::ui::frame::sidebar::{CAPTION_HEIGHT, LINE_HEIGHT, ListSpec, SECTION_GAP, host_labels};
use crate::ui::frame::{FrameLayout, HostsView};
use crate::ui::metrics::UiMetrics;

pub(in crate::ui::frame::editor) struct ConnectDialog<'a> {
    pub(in crate::ui::frame::editor) view: HostsView<'a>,
    pub(in crate::ui::frame::editor) strings: &'static Strings,
}

impl FrameLayout {
    pub(in crate::ui::frame::editor) fn connect_popup(
        &self,
        text: &mut TextSystem,
        dialog: &ConnectDialog,
        editor: EditorFrame,
    ) -> EditorFrame {
        let metrics = self.metrics;
        let ConnectDialog { view, strings } = *dialog;
        let form_space = form_height(&metrics);
        let other = form_space + list_chrome(&metrics);
        let shown = self.listed_rows(view.hosts.len(), other);
        let content = list_height(&metrics, shown) + form_space;
        let (popup, mut column) = self.open_popup(text, strings.ssh_connection, content);
        let mut sections = Vec::new();
        let mut list = None;
        if shown > 0 {
            sections.push(column.caption(strings.servers));
            let labels = host_labels(view.hosts);
            let spec = ListSpec {
                labels: &labels,
                first: view.first,
                shown,
                pickable: true,
                forgets: false,
            };
            list = Some(column.host_list(&spec));
            sections.push(column.section(form_title(strings, view.changing)));
        }
        let form = view.form.map(|form| {
            let area = Rect::new(column.inner.x, column.y, column.inner.width, form_space);
            let spec = FormSpec {
                area,
                form,
                actions: &[FormAction::Connect],
                strings,
            };
            place_form(column.text, &metrics, &spec)
        });
        EditorFrame {
            popup: Some(popup),
            sections,
            list,
            form,
            ..editor
        }
    }
}

fn list_chrome(metrics: &UiMetrics) -> f32 {
    2.0 * metrics.px(CAPTION_HEIGHT) + metrics.px(SECTION_GAP)
}

fn list_height(metrics: &UiMetrics, listed: usize) -> f32 {
    match listed {
        0 => 0.0,
        _ => list_chrome(metrics) + listed as f32 * metrics.px(LINE_HEIGHT),
    }
}
