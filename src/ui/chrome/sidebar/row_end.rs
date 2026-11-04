use super::super::{Icon, Painter};
use crate::theme::Rgba;
use crate::ui::frame::{SidebarItem, SidebarRow};
use crate::ui::hit::Hit;

pub(super) fn draw_row_end(painter: &mut Painter, row: &SidebarRow, glow: f32, ink: Option<Rgba>) {
    painter.scene.push_opacity(glow);
    painter.small_inked(row.close, Hit::RowClose(row.item), Icon::Close, ink);
    if let (SidebarItem::Instance(id), Some(fold)) = (row.item, row.fold) {
        let icon = Icon::Fold { folded: row.folded };
        painter.small_inked(fold, Hit::RowFold(id), icon, ink);
    }
    painter.scene.pop_opacity();
    if let Some(count) = &row.count {
        let x = row.close.right() - count.width;
        painter.scene.push_opacity(1.0 - glow);
        let faint = ink.unwrap_or(painter.theme.text_faint);
        painter.label(count, x, row.rect, faint);
        painter.scene.pop_opacity();
    }
}
