use crate::geometry::{Point, Rect};
use crate::theme::{Emblem, Swatch};
use crate::ui::display::PanelSide;
use crate::ui::frame::FrameInput;
use crate::ui::metrics::UiMetrics;
use crate::workspace::InstanceId;

const SLOT_GAP: f32 = 4.0;
const BESIDE_GAP: f32 = 4.0;

#[derive(Clone, Debug, PartialEq)]
pub struct RailSlot {
    pub id: InstanceId,
    pub name: String,
    pub rect: Rect,
    pub icon: Rect,
    pub active: bool,
    pub color: Option<Swatch>,
    pub emblem: Emblem,
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

    pub fn beside(&self, metrics: &UiMetrics, id: InstanceId, width: f32) -> Option<Rect> {
        let slot = self.slots.iter().find(|slot| slot.id == id)?;
        Some(self.next_to(metrics, slot.rect, width))
    }

    pub fn next_to(&self, metrics: &UiMetrics, anchor: Rect, width: f32) -> Rect {
        let height = metrics.px(UiMetrics::SIDEBAR_ROW);
        let gap = metrics.px(BESIDE_GAP);
        let x = match metrics.panel {
            PanelSide::Left => self.rect.right() + gap,
            PanelSide::Right => self.rect.x - gap - width,
        };
        Rect::new(x, (anchor.center().y - height / 2.0).round(), width, height)
    }
}

pub(crate) fn layout_rail(input: &FrameInput, rect: Rect) -> RailFrame {
    let (metrics, workspace) = (&input.metrics, input.workspace);
    let size = metrics.px(UiMetrics::BUTTON);
    let icon = metrics.px(UiMetrics::ICON);
    let rail = metrics.px(UiMetrics::RAIL_WIDTH);
    let header = metrics.spaced(UiMetrics::SIDEBAR_HEADER);
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
                name: instance
                    .shown_name(input.strings.instance_name)
                    .into_owned(),
                rect: slot,
                icon: slot.inset_by((size - icon) / 2.0, (size - icon) / 2.0),
                active: index == workspace.active,
                color: instance.color,
                emblem: workspace.shown_emblem(instance),
            }
        })
        .collect();
    RailFrame { rect, slots }
}
