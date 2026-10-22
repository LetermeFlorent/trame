mod editor;
pub mod hit;
mod label;
mod marks;
mod panes;
mod sidebar;
mod titlebar;

pub use editor::{EditAnchor, EditTarget, EditorFrame, EditorRequest, SwatchSlot};
pub use label::fit_label;
pub use panes::{PaneFrame, TabFrame, TabStrip};
pub use sidebar::{RailFrame, SidebarFrame, SidebarItem, SidebarRow};
pub use titlebar::{TitleBarFrame, WindowControls};

use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::TextSystem;
use crate::ui::metrics::UiMetrics;
use crate::ui::motion::Motion;
use crate::workspace::{Divider, PaneId, Workspace};

pub struct FrameInput<'a> {
    pub workspace: &'a Workspace,
    pub width: f32,
    pub height: f32,
    pub metrics: UiMetrics,
    pub maximized: bool,
    pub title: &'a dyn Fn(SessionId) -> String,
}

#[derive(Clone, Debug)]
pub struct FrameLayout {
    pub bounds: Rect,
    pub metrics: UiMetrics,
    pub title_bar: TitleBarFrame,
    pub side: Rect,
    pub reveal: f32,
    pub sidebar: Option<SidebarFrame>,
    pub rail: Option<RailFrame>,
    pub content: Rect,
    pub panes: Vec<PaneFrame>,
    pub dividers: Vec<Divider>,
    pub editor: Option<EditorFrame>,
}

impl FrameLayout {
    pub fn compute(text: &mut TextSystem, input: &FrameInput) -> Self {
        Self::compute_with(text, input, &mut Motion::default())
    }

    pub fn compute_with(text: &mut TextSystem, input: &FrameInput, motion: &mut Motion) -> Self {
        let metrics = input.metrics;
        let bounds = Rect::new(0.0, 0.0, input.width, input.height);
        let (bar, rest) = bounds.split_top(metrics.px(UiMetrics::TITLE_BAR));
        let title_input = titlebar::TitleInput {
            workspace: input.workspace,
            maximized: input.maximized,
            title: input.title,
        };
        let title_bar = titlebar::layout_title_bar(text, &metrics, bar, title_input);
        let open = input.workspace.sidebar_open;
        let reveal = motion.sidebar(open);
        let rail_width = metrics.px(UiMetrics::RAIL_WIDTH);
        let full_width = metrics.px(UiMetrics::SIDEBAR_WIDTH);
        let visible_width = (rail_width + (full_width - rail_width) * reveal).round();
        let (side, _) = rest.split_left(visible_width);
        let (_, content) = rest.split_left(if open { full_width } else { rail_width });
        let mut sidebar = (reveal > 0.0).then(|| {
            let rect = Rect::new(side.right() - full_width, side.y, full_width, side.height);
            sidebar::layout_sidebar(text, &metrics, rect, input.workspace, input.title)
        });
        let rail = (reveal < 1.0).then(|| sidebar::layout_rail(&metrics, side));
        let mut context = panes::PaneContext {
            text,
            metrics,
            title: input.title,
            motion,
        };
        let (mut panes, dividers) = match input.workspace.active_instance() {
            Some(instance) => panes::layout_panes(&mut context, content, instance),
            None => (Vec::new(), Vec::new()),
        };
        marks::apply(input.workspace, sidebar.as_mut(), &mut panes);
        Self {
            bounds,
            metrics,
            title_bar,
            side,
            reveal,
            sidebar,
            rail,
            content,
            panes,
            dividers,
            editor: None,
        }
    }

    pub fn pane(&self, id: PaneId) -> Option<&PaneFrame> {
        self.panes.iter().find(|pane| pane.id == id)
    }
}
