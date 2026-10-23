use crate::ui::chrome::{Icon, Painter, Press};
use crate::ui::frame::{HostList, HostRow, ListControl, ScrollBar};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

pub(in crate::ui::chrome::overlay) fn draw_host_list(painter: &mut Painter, list: &HostList) {
    for row in &list.rows {
        draw_row(painter, row, list.pickable);
    }
    if let Some(bar) = list.bar {
        draw_bar(painter, bar);
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

fn draw_bar(painter: &mut Painter, bar: ScrollBar) {
    let theme = painter.theme;
    let radius = bar.track.width / 2.0;
    if painter.state(Hit::List(ListControl::Track)) != Press::Idle {
        painter.scene.fill(bar.track, theme.hover, radius);
    }
    let thumb = match painter.state(Hit::List(ListControl::Thumb)) {
        Press::Idle => theme.border_strong,
        _ => theme.text_faint,
    };
    painter.scene.fill(bar.thumb, thumb, radius);
}
