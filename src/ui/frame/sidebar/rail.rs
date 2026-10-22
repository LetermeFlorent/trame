use crate::geometry::{Point, Rect};
use crate::theme::Swatch;
use crate::ui::metrics::UiMetrics;
use crate::workspace::{InstanceId, Workspace};

const SLOT_GAP: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RailSlot {
    pub id: InstanceId,
    pub rect: Rect,
    pub icon: Rect,
    pub active: bool,
    pub color: Option<Swatch>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RailFrame {
    pub rect: Rect,
    pub slots: Vec<RailSlot>,
}

impl RailFrame {
    pub fn slot_at(&self, point: Point) -> Option<&RailSlot> {
        self.slots.iter().find(|slot| slot.rect.contains(point))
    }
}

pub(crate) fn layout_rail(metrics: &UiMetrics, workspace: &Workspace, rect: Rect) -> RailFrame {
    let size = metrics.px(UiMetrics::BUTTON);
    let icon = metrics.px(UiMetrics::ICON);
    let rail = metrics.px(UiMetrics::RAIL_WIDTH);
    let header = metrics.px(UiMetrics::SIDEBAR_HEADER);
    let step = size + metrics.px(SLOT_GAP);
    let x = rect.x + (rail - size) / 2.0;
    let top = rect.y + (header - size) / 2.0;
    let slots = workspace
        .instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            let slot = Rect::new(x, top + index as f32 * step, size, size).snapped();
            RailSlot {
                id: instance.id,
                rect: slot,
                icon: slot.inset_by((size - icon) / 2.0, (size - icon) / 2.0),
                active: index == workspace.active,
                color: instance.color,
            }
        })
        .collect();
    RailFrame { rect, slots }
}
