use super::EditorFrame;
use super::emblems::{emblem_row, emblem_slots};
use super::form::{form_height, place_form};
use super::row::{RowSize, swatch_row, swatch_slots};
use crate::geometry::Rect;
use crate::locale::Strings;
use crate::preferences::{ConnectForm, SshHost};
use crate::text::TextSystem;
use crate::theme::{Emblem, Swatch};
use crate::ui::frame::sidebar::{CAPTION_HEIGHT, LINE_HEIGHT, SECTION_GAP};
use crate::ui::frame::{FrameLayout, PopupFrame};
use crate::ui::metrics::UiMetrics;

pub(super) struct ConnectDialog<'a> {
    pub(super) form: &'a ConnectForm,
    pub(super) hosts: &'a [SshHost],
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
        let strings = dialog.strings;
        let form_space = form_height(&metrics);
        let listed = self.listed_hosts(dialog.hosts.len(), form_space);
        let (popup, mut column) = self.open_popup(
            text,
            strings.ssh_connection,
            list_height(&metrics, listed) + form_space,
        );
        let mut sections = Vec::new();
        let mut hosts = Vec::new();
        if listed > 0 {
            sections.push(column.caption(strings.servers));
            for host in &dialog.hosts[..listed] {
                hosts.push(column.line(&host.label(), LINE_HEIGHT, metrics.label()));
            }
            sections.push(column.section(strings.new_connection));
        }
        let area = Rect::new(column.inner.x, column.y, column.inner.width, form_space);
        let form = place_form(column.text, &metrics, area, dialog.form, strings);
        EditorFrame {
            popup: Some(popup),
            sections,
            hosts,
            form: Some(form),
            ..editor
        }
    }

    fn listed_hosts(&self, count: usize, form_space: f32) -> usize {
        if count == 0 {
            return 0;
        }
        let room = self.popup_room() - form_space - list_chrome(&self.metrics);
        let fits = (room / self.metrics.px(LINE_HEIGHT)).floor().max(0.0) as usize;
        fits.min(count)
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
