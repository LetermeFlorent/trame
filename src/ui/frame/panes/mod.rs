mod dividers;
mod entry;
mod tabs;
mod widths;

use std::collections::HashMap;

pub use tabs::{TabFrame, TabStrip};

use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::{CellMetrics, TextSystem};
use crate::ui::metrics::UiMetrics;
use crate::ui::motion::{Motion, MotionKey};
use crate::workspace::{Divider, Instance, PaneId, PaneLayout};

pub(super) struct PaneContext<'a> {
    pub(super) text: &'a mut TextSystem,
    pub(super) metrics: UiMetrics,
    pub(super) title: &'a dyn Fn(SessionId) -> String,
    pub(super) motion: &'a mut Motion,
}

#[derive(Clone, Debug)]
pub struct PaneFrame {
    pub id: PaneId,
    pub rect: Rect,
    pub focused: bool,
    pub strip: TabStrip,
    pub body: Rect,
    pub terminal: Rect,
    pub grid_area: Rect,
    pub tabs: Vec<SessionId>,
    pub active: Option<SessionId>,
}

impl PaneFrame {
    pub fn grid_size(&self, cell: &CellMetrics) -> (usize, usize) {
        let rows = (self.grid_area.height / cell.height).floor().max(1.0) as usize;
        let cols = (self.grid_area.width / cell.width).floor().max(1.0) as usize;
        (rows, cols)
    }
}

struct Placement {
    id: PaneId,
    rect: Rect,
    target: Rect,
    known: bool,
}

pub(super) fn layout_panes(
    cx: &mut PaneContext,
    content: Rect,
    instance: Option<&Instance>,
) -> (Vec<PaneFrame>, Vec<Divider>) {
    let Some(instance) = instance else {
        return (Vec::new(), Vec::new());
    };
    let layout = PaneLayout::compute(&instance.root, content, cx.metrics.line());
    let entries = entry::entries(&layout, cx.motion);
    let mut shown = HashMap::new();
    let mut panes = Vec::with_capacity(layout.panes.len());
    for (id, target) in &layout.panes {
        let key = MotionKey::Pane(*id);
        let known = cx.motion.knows(key);
        let rect = cx.motion.place(key, *target, entries.get(id).copied());
        shown.insert(*id, rect);
        let placement = Placement {
            id: *id,
            rect,
            target: *target,
            known,
        };
        panes.extend(pane_frame(cx, instance, placement));
    }
    let targets: HashMap<PaneId, Rect> = layout.panes.iter().copied().collect();
    let dividers = layout
        .dividers
        .iter()
        .map(|divider| dividers::follow(divider, &instance.root, &shown, &targets))
        .collect();
    (panes, dividers)
}

fn pane_frame(
    cx: &mut PaneContext,
    instance: &Instance,
    placement: Placement,
) -> Option<PaneFrame> {
    let pane = instance.panes.get(&placement.id)?;
    let bar_height = cx.metrics.px(UiMetrics::TAB_BAR);
    let padding_x = cx.metrics.px(UiMetrics::TERMINAL_PAD_X);
    let padding_y = cx.metrics.px(UiMetrics::TERMINAL_PAD_Y);
    let (bar, body) = placement.rect.split_top(bar_height);
    let (target_bar, target_body) = placement.target.split_top(bar_height);
    let strip_placement = tabs::StripPlacement {
        pane_id: placement.id,
        pane,
        bar,
        target: target_bar,
        grows: placement.known,
    };
    Some(PaneFrame {
        id: placement.id,
        rect: placement.rect,
        focused: placement.id == instance.active_pane,
        strip: tabs::layout_strip(cx, strip_placement),
        body,
        terminal: body.inset_by(padding_x, padding_y),
        grid_area: target_body.inset_by(padding_x, padding_y),
        tabs: pane.tabs.clone(),
        active: pane.active_tab(),
    })
}
