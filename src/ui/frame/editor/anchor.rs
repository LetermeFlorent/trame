use super::EditTarget;
use crate::geometry::Rect;
use crate::ui::frame::{FrameLayout, SidebarItem};
use crate::ui::metrics::UiMetrics;
use crate::workspace::PaneId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditAnchor {
    Strip(PaneId),
    Sidebar,
    Rail,
}

impl FrameLayout {
    pub(super) fn anchor_rect(&self, anchor: EditAnchor, target: EditTarget) -> Option<Rect> {
        match (anchor, target) {
            (EditAnchor::Strip(pane), EditTarget::Tab(id)) => self
                .pane(pane)?
                .strip
                .tabs
                .iter()
                .find(|tab| tab.id == id)
                .map(|tab| tab.rect),
            (EditAnchor::Sidebar, target) => {
                let item = match target {
                    EditTarget::Instance(id) => SidebarItem::Instance(id),
                    EditTarget::Tab(id) => SidebarItem::Tab(id),
                };
                let sidebar = self.sidebar.as_ref()?;
                sidebar
                    .rows
                    .iter()
                    .find(|row| row.item == item)
                    .map(|row| row.rect)
            }
            (EditAnchor::Rail, EditTarget::Instance(id)) => {
                let width = self.metrics.px(UiMetrics::SIDEBAR_WIDTH);
                self.rail.as_ref()?.beside(&self.metrics, id, width)
            }
            _ => None,
        }
    }
}
