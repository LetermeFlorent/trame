use trame::geometry::Point;
use trame::ui::Hit;
use trame::ui::frame::{ListControl, ScrollBar};

use crate::app::running::{Drag, Running};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app::running) enum ListKind {
    Hosts,
    Sidebar,
}

impl Running {
    pub(in crate::app::running) fn list_control(&mut self, control: ListControl) {
        if self.custom_control(control) || self.shortcut_control(control) {
            return self.changed();
        }
        match control {
            ListControl::Pick(index) => self.connect_listed(index),
            ListControl::Change(index) => self.change_host(index),
            ListControl::Forget(index) => self.forget_listed(index),
            ListControl::Thumb => self.drag_pressed(Hit::List(control)),
            ListControl::Track => self.page_list(ListKind::Hosts),
        }
        self.changed();
    }

    pub(in crate::app::running) fn page_list(&mut self, list: ListKind) {
        let y = self.pointer.position.map(|point| point.y);
        if let (Some(bar), Some(y)) = (self.list_bar(list), y) {
            self.scroll_list_to(list, bar.paged(y));
        }
    }

    pub(in crate::app::running) fn list_drag(&self, list: ListKind, origin: Point) -> Option<Drag> {
        let grab = origin.y - self.list_bar(list)?.thumb.y;
        Some(Drag::List(list, grab))
    }

    pub(in crate::app::running) fn drag_list(&mut self, list: ListKind, thumb_top: f32) {
        if let Some(bar) = self.list_bar(list) {
            self.scroll_list_to(list, bar.first_at(thumb_top));
        }
    }

    pub(in crate::app::running) fn list_under_pointer(&self, list: ListKind) -> Option<ScrollBar> {
        let point = self.pointer.position?;
        let area = match list {
            ListKind::Hosts => self.frame.host_list()?.rect,
            ListKind::Sidebar => self.frame.sidebar.as_ref()?.list,
        };
        self.list_bar(list).filter(|_| area.contains(point))
    }

    pub(in crate::app::running) fn scroll_list_to(&mut self, list: ListKind, first: usize) {
        let shown = match list {
            ListKind::Hosts => self.list_first_mut(),
            ListKind::Sidebar => Some(&mut self.workspace.sidebar_first),
        };
        let Some(shown) = shown else {
            return;
        };
        if *shown != first {
            *shown = first;
            self.changed();
        }
    }

    fn list_bar(&self, list: ListKind) -> Option<ScrollBar> {
        match list {
            ListKind::Hosts => self.frame.host_list()?.bar,
            ListKind::Sidebar => self.frame.sidebar.as_ref()?.bar,
        }
    }
}
