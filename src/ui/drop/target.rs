use super::zone::DropZone;
use crate::geometry::{Point, Rect};
use crate::ui::frame::{FrameLayout, PaneFrame, SidebarFrame, SidebarItem};
use crate::ui::metrics::UiMetrics;
use crate::workspace::{DropTarget, InstanceId};

const MARKER_WIDTH: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DropMark {
    Area(Rect),
    Marker(Rect),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropSpot {
    pub target: DropTarget,
    pub mark: DropMark,
}

impl FrameLayout {
    pub fn drop_at(&self, point: Point, instance: InstanceId) -> Option<DropSpot> {
        if let Some(sidebar) = self
            .sidebar
            .as_ref()
            .filter(|side| side.rect.contains(point))
        {
            return sidebar_drop(sidebar, point);
        }
        let pane = self.panes.iter().find(|pane| pane.rect.contains(point))?;
        match pane.strip.rect.contains(point) {
            true => Some(strip_drop(&self.metrics, pane, point, instance)),
            false => body_drop(pane, point, instance),
        }
    }
}

fn sidebar_drop(sidebar: &SidebarFrame, point: Point) -> Option<DropSpot> {
    if sidebar.new_instance.contains(point) {
        return Some(DropSpot {
            target: DropTarget::NewInstance,
            mark: DropMark::Area(sidebar.new_instance),
        });
    }
    let position = sidebar
        .rows
        .iter()
        .position(|row| row.rect.contains(point))?;
    let owner = sidebar.rows[..=position]
        .iter()
        .rev()
        .find_map(|row| match row.item {
            SidebarItem::Instance(id) => Some(id),
            SidebarItem::Tab(_) => None,
        })?;
    Some(DropSpot {
        target: DropTarget::Instance(owner),
        mark: DropMark::Area(sidebar.rows[position].rect),
    })
}

fn strip_drop(
    metrics: &UiMetrics,
    pane: &PaneFrame,
    point: Point,
    instance: InstanceId,
) -> DropSpot {
    let strip = &pane.strip;
    let index = strip.insertion_index(point.x);
    let x = strip
        .tabs
        .get(index)
        .map(|tab| tab.rect.x)
        .or_else(|| strip.tabs.last().map(|tab| tab.rect.right()))
        .unwrap_or(strip.tabs_area.x);
    let width = metrics.px(MARKER_WIDTH);
    let left = (x - width / 2.0).round().max(strip.tabs_area.x);
    DropSpot {
        target: DropTarget::Tabs {
            instance,
            pane: pane.id,
            index: Some(index),
        },
        mark: DropMark::Marker(Rect::new(left, strip.rect.y, width, strip.rect.height)),
    }
}

fn body_drop(pane: &PaneFrame, point: Point, instance: InstanceId) -> Option<DropSpot> {
    let zone = DropZone::at(pane.body, point)?;
    let target = match zone {
        DropZone::Center => DropTarget::Swap {
            instance,
            pane: pane.id,
        },
        DropZone::Edge(edge) => DropTarget::Split {
            instance,
            pane: pane.id,
            edge,
        },
    };
    Some(DropSpot {
        target,
        mark: DropMark::Area(zone.area(pane.body)),
    })
}
