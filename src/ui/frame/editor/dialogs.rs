use super::EditorFrame;
use super::emblems::{emblem_row, emblem_slots};
use super::form::{FormAction, FormSpec, form_height, form_title, place_form};
use super::row::{RowSize, swatch_row, swatch_slots};
use crate::geometry::Rect;
use crate::locale::Strings;
use crate::text::TextSystem;
use crate::theme::{Emblem, Swatch};
use crate::ui::frame::sidebar::{CAPTION_HEIGHT, LINE_HEIGHT, ListSpec, SECTION_GAP};
use crate::ui::frame::{FrameLayout, HostsView, PopupFrame};
use crate::ui::metrics::UiMetrics;

const MAX_LISTED: usize = 6;

pub(super) struct ConnectDialog<'a> {
    pub(super) view: HostsView<'a>,
    pub(super) strings: &'static Strings,
}

impl FrameLayout {
    pub(super) fn color_popup(
        &self,
        text: &mut TextSystem,
        strings: &Strings,
        selected: Option<Swatch>,
        editor: EditorFrame,
    ) -> EditorFrame {
        let (popup, strip) = self.row_popup(text, strings.change_color, swatch_row(&self.metrics));
        EditorFrame {
            popup: Some(popup),
            slots: swatch_slots(&self.metrics, strip),
            strip,
            selected,
            ..editor
        }
    }

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

    pub(super) fn connect_popup(
        &self,
        text: &mut TextSystem,
        dialog: &ConnectDialog,
        editor: EditorFrame,
    ) -> EditorFrame {
        let metrics = self.metrics;
        let ConnectDialog { view, strings } = *dialog;
        let form_space = form_height(&metrics);
        let shown = self.listed_hosts(view.hosts.len(), form_space);
        let content = list_height(&metrics, shown) + form_space;
        let (popup, mut column) = self.open_popup(text, strings.ssh_connection, content);
        let mut sections = Vec::new();
        let mut list = None;
        if shown > 0 {
            sections.push(column.caption(strings.servers));
            let spec = ListSpec {
                hosts: view.hosts,
                first: view.first,
                shown,
                manage: false,
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

    fn listed_hosts(&self, count: usize, form_space: f32) -> usize {
        if count == 0 {
            return 0;
        }
        let room = self.popup_room() - form_space - list_chrome(&self.metrics);
        let fits = (room / self.metrics.px(LINE_HEIGHT)).floor().max(0.0) as usize;
        fits.min(count).min(MAX_LISTED)
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
