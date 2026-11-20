use trame::geometry::Point;
use trame::ui::frame::ScrollBar;

use super::ListKind;
use crate::gnome::state::WindowState;
use crate::gnome::state::drag::Drag;
use crate::gnome::state::editing::EditMode;

impl WindowState {
    pub fn page_list(&mut self, list: ListKind) {
        if let (Some(bar), Some(p)) = (self.list_bar(list), self.pointer) {
            self.scroll_list_to(list, bar.paged(p.y));
        }
    }

    pub fn list_drag(&self, list: ListKind, origin: Point) -> Option<Drag> {
        Some(Drag::List(list, origin.y - self.list_bar(list)?.thumb.y))
    }

    pub fn drag_list(&mut self, list: ListKind, top: f32) {
        if let Some(bar) = self.list_bar(list) {
            self.scroll_list_to(list, bar.first_at(top));
        }
    }

    pub fn list_under_pointer(&self, list: ListKind) -> Option<ScrollBar> {
        let p = self.pointer?;
        let area = match list {
            ListKind::Hosts => self.frame.host_list()?.rect,
            ListKind::Sidebar => self.frame.sidebar.as_ref()?.list,
        };
        self.list_bar(list).filter(|_| area.contains(p))
    }

    pub fn scroll_list_to(&mut self, list: ListKind, first: usize) {
        let changed = match self.list_first_mut(list) {
            Some(slot) if *slot != first => {
                *slot = first;
                true
            }
            _ => false,
        };
        if changed {
            self.changed();
        }
    }

    fn list_first_mut(&mut self, list: ListKind) -> Option<&mut usize> {
        if list == ListKind::Sidebar {
            return Some(&mut self.desk.workspace.sidebar_first);
        }
        match self.editing.as_mut().map(|e| &mut e.mode) {
            Some(EditMode::Color { first }) => Some(first),
            Some(EditMode::Form(panel)) => Some(&mut panel.first),
            _ => match self.settings.as_mut() {
                Some(s) => s
                    .hosts
                    .as_mut()
                    .map(|h| &mut h.first)
                    .or_else(|| s.keys.as_mut().map(|k| &mut k.first)),
                None => None,
            },
        }
    }

    fn list_bar(&self, list: ListKind) -> Option<ScrollBar> {
        match list {
            ListKind::Hosts => self.frame.host_list()?.bar,
            ListKind::Sidebar => self.frame.sidebar.as_ref()?.bar,
        }
    }
}
