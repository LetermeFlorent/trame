use crate::geometry::Rect;
use crate::ui::frame::sidebar::scroll::{BAR_GAP, BAR_WIDTH, ListWindow, ScrollBar};
use crate::ui::frame::sidebar::settings::column::{BUTTON_GAP, Column, LINE_HEIGHT};

use super::{HostList, HostRow};

pub(in crate::ui::frame) struct ListSpec<'a> {
    pub(in crate::ui::frame) labels: &'a [String],
    pub(in crate::ui::frame) first: usize,
    pub(in crate::ui::frame) shown: usize,
    pub(in crate::ui::frame) pickable: bool,
    pub(in crate::ui::frame) forgets: bool,
}

impl Column<'_> {
    pub(in crate::ui::frame) fn host_list(&mut self, spec: &ListSpec) -> HostList {
        let count = spec.labels.len();
        let shown = spec.shown.min(count);
        let last_first = count - shown;
        let first = spec.first.min(last_first);
        let height = shown as f32 * self.metrics.px(LINE_HEIGHT);
        let rect = self.band(height);
        let window = ListWindow {
            first,
            shown,
            count,
        };
        let bar = (last_first > 0).then(|| ScrollBar::fit(&self.metrics, rect, window));
        let inner = self.inner;
        if bar.is_some() {
            self.inner.width -= self.metrics.px(BAR_WIDTH + BAR_GAP);
        }
        let rows = (first..first + shown)
            .map(|index| self.host_row(index, &spec.labels[index], spec.forgets))
            .collect();
        self.inner = inner;
        HostList {
            rect,
            rows,
            bar,
            pickable: spec.pickable,
        }
    }

    fn host_row(&mut self, index: usize, label: &str, forgets: bool) -> HostRow {
        let buttons = if forgets { 2.0 } else { 1.0 };
        let style = self.metrics.label();
        let line = self.line_beside(label, LINE_HEIGHT, style, buttons);
        let end = self.end_button(&line);
        let before = Rect {
            x: end.x - end.width - self.metrics.px(BUTTON_GAP),
            ..end
        };
        let (change, forget) = match forgets {
            true => (before, Some(end)),
            false => (end, None),
        };
        HostRow {
            index,
            line,
            change,
            forget,
        }
    }
}
