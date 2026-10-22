mod editor;
pub mod hit;
mod label;
mod marks;
mod panes;
mod sidebar;
mod titlebar;

pub use editor::{EditAnchor, EditTarget, EditorFrame, EditorRequest, EmblemSlot, SwatchSlot};
pub use label::fit_label;
pub use panes::{PaneFrame, TabFrame, TabStrip};
pub use sidebar::{
    FooterFrame, RailFrame, RailSlot, SettingControl, SettingLine, SettingsFrame, SidebarFrame,
    SidebarItem, SidebarRow, Stepper, dragged_width, shown_width,
};
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
    pub footer: FooterFrame,
    pub content: Rect,
    pub panes: Vec<PaneFrame>,
    pub dividers: Vec<Divider>,
    pub editor: Option<EditorFrame>,
    pub settings: Option<SettingsFrame>,
    pub blocked: bool,
}

impl FrameLayout {
    pub fn compute(text: &mut TextSystem, input: &FrameInput) -> Self {
        Self::compute_with(text, input, &mut Motion::default())
    }

    pub fn compute_with(text: &mut TextSystem, input: &FrameInput, motion: &mut Motion) -> Self {
        let metrics = input.metrics;
        let bounds = Rect::new(0.0, 0.0, input.width, input.height);
        let (bar, rest) = bounds.split_top(metrics.px(UiMetrics::TITLE_BAR));
        let title_bar = titlebar::layout_title_bar(text, input, bar);
        let reveal = motion.sidebar(input.workspace.sidebar_open);
        let columns = Columns::split(input, rest, reveal);
        let (side, content) = (columns.side, columns.content);
        let footer = sidebar::layout_footer(&metrics, side);
        let mut sidebar = (reveal > 0.0)
            .then(|| sidebar::layout_sidebar(text, input, footer.above(columns.sidebar)));
        let rail = (reveal < 1.0)
            .then(|| sidebar::layout_rail(&metrics, input.workspace, footer.above(side)));
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
            footer,
            content,
            panes,
            dividers,
            editor: None,
            settings: None,
            blocked: false,
        }
    }

    pub fn sidebar_edge(&self) -> Rect {
        let line = self.metrics.line();
        let grab = self.metrics.px(UiMetrics::DIVIDER_GRAB);
        Rect::new(
            self.side.right() - line,
            self.side.y,
            line,
            self.side.height,
        )
        .inset_by(-grab, 0.0)
    }

    pub fn pane(&self, id: PaneId) -> Option<&PaneFrame> {
        self.panes.iter().find(|pane| pane.id == id)
    }
}

struct Columns {
    side: Rect,
    sidebar: Rect,
    content: Rect,
}

impl Columns {
    fn split(input: &FrameInput, rest: Rect, reveal: f32) -> Self {
        let metrics = input.metrics;
        let rail_width = metrics.px(UiMetrics::RAIL_WIDTH);
        let full_width = metrics
            .px(shown_width(input.workspace.sidebar_width))
            .round();
        let visible_width = (rail_width + (full_width - rail_width) * reveal).round();
        let (side, _) = rest.split_left(visible_width);
        let shown = if input.workspace.sidebar_open {
            full_width
        } else {
            rail_width
        };
        let (_, content) = rest.split_left(shown);
        Self {
            side,
            sidebar: Rect::new(side.right() - full_width, side.y, full_width, side.height),
            content,
        }
    }
}
