mod editor;
pub mod hit;
mod label;
mod marks;
mod panes;
mod sidebar;
mod titlebar;

pub use editor::{
    ColorsView, EditAnchor, EditKind, EditTarget, EditorFrame, EditorRequest, EmblemSlot,
    EmblemView, FieldFrame, FormAction, FormButton, FormFrame, FormRow, MenuEntry, MenuFrame,
    MenuRequest, MenuRow, PickerFrame, PickerPart, PickerView, SECRET_MASK, SwatchSlot,
};
pub use label::{fit_label, fit_label_start, fit_text_start};
pub use panes::{PaneFrame, TabFrame, TabStrip};
pub use sidebar::{
    Chip, ChoiceRow, ConnectionsPage, FooterButton, FooterFrame, FormPage, HostList, HostRow,
    HostsView, InstancesPage, KeyControl, KeyDetail, KeyLine, KeyNotice, ListControl, MessageFrame,
    MessageRequest, NumberChoice, PopupFrame, RailFrame, RailSlot, ScrollBar, SettingControl,
    SettingLine, SettingsFrame, SettingsPage, SettingsRequest, SettingsTab, ShortcutsPage,
    ShortcutsView, SidebarFrame, SidebarItem, SidebarRow, Stepper, ToggleRow, dragged_width,
    key_label, shown_width,
};
pub use titlebar::{TitleBarFrame, WindowControls};

use std::path::Path;

use crate::geometry::Rect;
use crate::locale::Strings;
use crate::session::SessionId;
use crate::text::TextSystem;
use crate::ui::display::PanelSide;
use crate::ui::metrics::UiMetrics;
use crate::ui::motion::Motion;
use crate::workspace::{Divider, PaneId, Workspace};

pub struct FrameInput<'a> {
    pub workspace: &'a Workspace,
    pub width: f32,
    pub height: f32,
    pub metrics: UiMetrics,
    pub maximized: bool,
    pub fullscreen: bool,
    pub title: &'a dyn Fn(SessionId) -> String,
    pub folder: Option<&'a Path>,
    pub update: Option<&'a str>,
    pub strings: &'static Strings,
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
    pub menu: Option<MenuFrame>,
    pub settings: Option<SettingsFrame>,
    pub message: Option<MessageFrame>,
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
        let columns = sidebar::Columns::split(input, rest, reveal);
        let (side, content) = (columns.side, columns.content);
        let footer = sidebar::layout_footer(text, input, side);
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
        let active = input.workspace.active_instance();
        let (mut panes, dividers) = panes::layout_panes(&mut context, content, active);
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
            menu: None,
            settings: None,
            message: None,
            blocked: false,
        }
    }

    pub fn sidebar_edge(&self) -> Rect {
        let line = self.metrics.line();
        let grab = self.metrics.px(UiMetrics::DIVIDER_GRAB);
        let x = match self.metrics.panel {
            PanelSide::Left => self.side.right() - line,
            PanelSide::Right => self.side.x,
        };
        Rect::new(x, self.side.y, line, self.side.height).inset_by(-grab, 0.0)
    }

    pub fn pane(&self, id: PaneId) -> Option<&PaneFrame> {
        self.panes.iter().find(|pane| pane.id == id)
    }
}
