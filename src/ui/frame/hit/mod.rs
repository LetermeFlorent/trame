mod editor;
mod zones;

use crate::geometry::Point;
use crate::preferences::FormField;
use crate::session::SessionId;
use crate::theme::{Emblem, Swatch};
use crate::ui::frame::{
    FormAction, FrameLayout, ListControl, MenuEntry, SettingControl, SidebarItem,
};
use crate::ui::metrics::UiMetrics;
use crate::workspace::{InstanceId, PaneId};
use editor::{hit_editor, hit_message};
use zones::{
    hit_menu, hit_pane, hit_settings, hit_sidebar, hit_title_bar, resize_edge, through_backdrop,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeEdge {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Resize(ResizeEdge),
    TitleBar,
    SidebarToggle,
    RailInstance(InstanceId),
    SidebarEdge,
    Minimize,
    Maximize,
    Close,
    Sidebar,
    NewInstance,
    Row(SidebarItem),
    RowClose(SidebarItem),
    TabBar(PaneId),
    Tab(PaneId, SessionId),
    TabClose(PaneId, SessionId),
    NewTab(PaneId),
    Split(PaneId),
    Divider(usize),
    Terminal(PaneId),
    EditField,
    EditStrip,
    EditSwatch(Option<Swatch>),
    EditEmblem(Emblem),
    EditClose,
    FormField(FormField),
    FormButton(FormAction),
    List(ListControl),
    Settings,
    Update,
    SettingsPanel,
    Setting(SettingControl),
    Menu,
    MenuEntry(MenuEntry),
    Dialog,
    Backdrop,
}

impl FrameLayout {
    pub fn hit(&self, point: Point, resizable: bool) -> Option<Hit> {
        if !self.bounds.contains(point) {
            return None;
        }
        let border = self.metrics.px(UiMetrics::RESIZE_BORDER);
        let hit = resizable
            .then(|| resize_edge(self.bounds, border, point))
            .flatten()
            .map(Hit::Resize)
            .or_else(|| self.overlay_hit(point))
            .or_else(|| self.chrome_hit(point))
            .or_else(|| self.content_hit(point));
        match self.blocked {
            true => Some(through_backdrop(hit)),
            false => hit,
        }
    }

    fn overlay_hit(&self, point: Point) -> Option<Hit> {
        let message = self.message.as_ref();
        let menu = || hit_menu(self.menu.as_ref()?, point);
        let editor = || hit_editor(self.editor.as_ref()?, point);
        let settings = || hit_settings(self.settings.as_ref()?, point);
        message
            .and_then(|message| hit_message(message, point))
            .or_else(menu)
            .or_else(editor)
            .or_else(settings)
    }

    fn chrome_hit(&self, point: Point) -> Option<Hit> {
        if self.title_bar.rect.contains(point) {
            return Some(hit_title_bar(&self.title_bar, point));
        }
        if self.sidebar_edge().contains(point) {
            return Some(Hit::SidebarEdge);
        }
        if self.footer.rect.contains(point) {
            return Some(self.footer.hit(point));
        }
        if let Some(sidebar) = self
            .sidebar
            .as_ref()
            .filter(|side| side.rect.contains(point))
        {
            return Some(hit_sidebar(sidebar, point));
        }
        let rail = self
            .rail
            .as_ref()
            .filter(|rail| rail.rect.contains(point))?;
        Some(
            rail.slot_at(point)
                .map_or(Hit::Sidebar, |slot| Hit::RailInstance(slot.id)),
        )
    }

    fn content_hit(&self, point: Point) -> Option<Hit> {
        if let Some(index) = self.divider_at(point) {
            return Some(Hit::Divider(index));
        }
        self.panes
            .iter()
            .find(|pane| pane.rect.contains(point))
            .map(|pane| hit_pane(pane, point))
    }

    pub fn divider_at(&self, point: Point) -> Option<usize> {
        let grab = self.metrics.px(UiMetrics::DIVIDER_GRAB);
        self.dividers
            .iter()
            .position(|divider| divider.widened(grab).contains(point))
    }
}
