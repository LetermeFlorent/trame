use crate::ui::chrome::sidebar::draw_bar;
use crate::ui::chrome::{Icon, Painter};
use crate::ui::frame::{HostList, HostRow, ListControl};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

pub(in crate::ui::chrome::overlay) fn draw_host_list(painter: &mut Painter, list: &HostList) {
    for row in &list.rows {
        draw_row(painter, row, list.pickable);
    }
    if let Some(bar) = list.bar {
        let track = Hit::List(ListControl::Track);
        draw_bar(painter, bar, track, Hit::List(ListControl::Thumb));
    }
}

fn draw_row(painter: &mut Painter, row: &HostRow, pickable: bool) {
    let theme = painter.theme;
    let color = match pickable {
        true => {
            let radius = painter.metrics.px(UiMetrics::RADIUS);
            let pick = Hit::List(ListControl::Pick(row.index));
            theme.muted_text(painter.press_fill(row.line.rect, pick, radius))
        }
        false => theme.text,
    };
    painter.setting_line(&row.line, color);
    let change = Hit::List(ListControl::Change(row.index));
    painter.small_button(row.change, change, Icon::Pencil);
    if let Some(rect) = row.forget {
        let forget = Hit::List(ListControl::Forget(row.index));
        painter.small_button(rect, forget, Icon::Close);
    }
}
