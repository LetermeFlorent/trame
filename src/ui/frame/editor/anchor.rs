use super::{EditTarget, EditorFrame};
use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::{TextLayout, TextStyle};
use crate::ui::frame::{FrameLayout, SidebarItem};
use crate::ui::metrics::UiMetrics;
use crate::workspace::PaneId;

const CLOSE_GAP: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditAnchor {
    Strip(PaneId),
    Sidebar,
    Rail,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LabelSpot {
    pub(super) x: f32,
    pub(super) right: f32,
    pub(super) style: TextStyle,
}

pub(super) struct AnchorSpot {
    pub(super) rect: Rect,
    pub(super) label: Option<LabelSpot>,
}

impl FrameLayout {
    pub(super) fn anchor_spot(&self, anchor: EditAnchor, target: EditTarget) -> Option<AnchorSpot> {
        match (anchor, target) {
            (EditAnchor::Strip(pane), EditTarget::Tab(id)) => {
                let strip = &self.pane(pane)?.strip;
                let tab = strip.tabs.iter().find(|tab| tab.id == id)?;
                Some(self.labelled_spot(tab.rect, &tab.label, tab.label_x, tab.close))
            }
            (EditAnchor::Sidebar, target) => {
                let item = SidebarItem::from(target);
                let rows = &self.sidebar.as_ref()?.rows;
                let row = rows.iter().find(|row| row.item == item)?;
                Some(self.labelled_spot(row.rect, &row.label, row.label_x, row.close))
            }
            (EditAnchor::Rail, EditTarget::Instance(id)) => {
                let width = self.metrics.px(UiMetrics::SIDEBAR_WIDTH);
                let rect = self.rail.as_ref()?.beside(&self.metrics, id, width)?;
                Some(AnchorSpot { rect, label: None })
            }
            _ => None,
        }
    }

    fn labelled_spot(&self, rect: Rect, label: &TextLayout, x: f32, close: Rect) -> AnchorSpot {
        let spot = LabelSpot {
            x,
            right: close.x - self.metrics.px(CLOSE_GAP),
            style: label.style,
        };
        AnchorSpot {
            rect,
            label: Some(spot),
        }
    }
}

impl From<EditTarget> for SidebarItem {
    fn from(target: EditTarget) -> Self {
        match target {
            EditTarget::Instance(id) => Self::Instance(id),
            EditTarget::Tab(id) => Self::Tab(id),
        }
    }
}

impl EditorFrame {
    pub fn renames_tab(&self, pane: PaneId, tab: SessionId) -> bool {
        self.edits_inline()
            && self.anchor == EditAnchor::Strip(pane)
            && self.target == EditTarget::Tab(tab)
    }

    pub fn renames_row(&self, item: SidebarItem) -> bool {
        self.edits_inline()
            && self.anchor == EditAnchor::Sidebar
            && SidebarItem::from(self.target) == item
    }

    fn edits_inline(&self) -> bool {
        self.field.as_ref().is_some_and(|field| !field.boxed)
    }
}
