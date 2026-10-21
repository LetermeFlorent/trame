mod label;
mod sidebar;
mod tabs;
mod titlebar;

pub use label::fit_label;
pub use sidebar::{RailFrame, SidebarFrame, SidebarItem, SidebarRow};
pub use tabs::{TabFrame, TabStrip};
pub use titlebar::{TitleBarFrame, WindowControls};

use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::{CellMetrics, TextSystem};
use crate::ui::metrics::UiMetrics;
use crate::workspace::{Divider, Instance, PaneId, PaneLayout, Workspace};

pub struct FrameInput<'a> {
    pub workspace: &'a Workspace,
    pub width: f32,
    pub height: f32,
    pub metrics: UiMetrics,
    pub maximized: bool,
    pub title: &'a dyn Fn(SessionId) -> String,
}

#[derive(Clone, Debug)]
pub struct PaneFrame {
    pub id: PaneId,
    pub rect: Rect,
    pub focused: bool,
    pub strip: TabStrip,
    pub body: Rect,
    pub terminal: Rect,
    pub tabs: Vec<SessionId>,
    pub active: Option<SessionId>,
}

impl PaneFrame {
    pub fn grid_size(&self, cell: &CellMetrics) -> (usize, usize) {
        let rows = (self.terminal.height / cell.height).floor().max(1.0) as usize;
        let cols = (self.terminal.width / cell.width).floor().max(1.0) as usize;
        (rows, cols)
    }
}

#[derive(Clone, Debug)]
pub struct FrameLayout {
    pub bounds: Rect,
    pub metrics: UiMetrics,
    pub title_bar: TitleBarFrame,
    pub sidebar: Option<SidebarFrame>,
    pub rail: Option<RailFrame>,
    pub content: Rect,
    pub panes: Vec<PaneFrame>,
    pub dividers: Vec<Divider>,
}

impl FrameLayout {
    pub fn compute(text: &mut TextSystem, input: &FrameInput) -> Self {
        let metrics = input.metrics;
        let bounds = Rect::new(0.0, 0.0, input.width, input.height);
        let (bar, rest) = bounds.split_top(metrics.px(UiMetrics::TITLE_BAR));
        let title_input = titlebar::TitleInput {
            workspace: input.workspace,
            maximized: input.maximized,
            title: input.title,
        };
        let title_bar = titlebar::layout_title_bar(text, &metrics, bar, title_input);
        let (sidebar, rail, content) = match input.workspace.sidebar_open {
            true => {
                let (side, content) = rest.split_left(metrics.px(UiMetrics::SIDEBAR_WIDTH));
                let frame =
                    sidebar::layout_sidebar(text, &metrics, side, input.workspace, input.title);
                (Some(frame), None, content)
            }
            false => {
                let (strip, content) = rest.split_left(metrics.px(UiMetrics::RAIL_WIDTH));
                (None, Some(sidebar::layout_rail(&metrics, strip)), content)
            }
        };
        let (panes, dividers) = match input.workspace.active_instance() {
            Some(instance) => layout_panes(text, &metrics, content, instance, input.title),
            None => (Vec::new(), Vec::new()),
        };
        Self {
            bounds,
            metrics,
            title_bar,
            sidebar,
            rail,
            content,
            panes,
            dividers,
        }
    }

    pub fn pane(&self, id: PaneId) -> Option<&PaneFrame> {
        self.panes.iter().find(|pane| pane.id == id)
    }
}

fn layout_panes(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    content: Rect,
    instance: &Instance,
    title: &dyn Fn(SessionId) -> String,
) -> (Vec<PaneFrame>, Vec<Divider>) {
    let layout = PaneLayout::compute(&instance.root, content, metrics.line());
    let panes = layout
        .panes
        .iter()
        .filter_map(|(id, rect)| {
            let pane = instance.panes.get(id)?;
            let (bar, body) = rect.split_top(metrics.px(UiMetrics::TAB_BAR));
            let padding_x = metrics.px(UiMetrics::TERMINAL_PAD_X);
            let padding_y = metrics.px(UiMetrics::TERMINAL_PAD_Y);
            Some(PaneFrame {
                id: *id,
                rect: *rect,
                focused: *id == instance.active_pane,
                strip: tabs::layout_strip(text, metrics, bar, pane, title),
                body,
                terminal: body.inset_by(padding_x, padding_y),
                tabs: pane.tabs.clone(),
                active: pane.active_tab(),
            })
        })
        .collect();
    (panes, layout.dividers)
}
