use trame::geometry::Point;
use trame::ui::Hit;
use trame::ui::frame::{ListControl, ScrollBar};

use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn list_control(&mut self, control: ListControl) {
        if self.custom_control(control) {
            return self.changed();
        }
        match control {
            ListControl::Pick(index) => self.connect_listed(index),
            ListControl::Change(index) => self.change_host(index),
            ListControl::Forget(index) => self.forget_listed(index),
            ListControl::Thumb => self.drag_pressed(Hit::List(control)),
            ListControl::Track => self.page_list(),
        }
        self.changed();
    }

    fn page_list(&mut self) {
        let y = self.pointer.position.map(|point| point.y);
        if let (Some(bar), Some(y)) = (self.list_bar(), y) {
            self.scroll_list_to(bar.paged(y));
        }
    }

    pub(in crate::app::running) fn list_grab(&self, origin: Point) -> Option<f32> {
        Some(origin.y - self.list_bar()?.thumb.y)
    }

    pub(in crate::app::running) fn drag_list(&mut self, thumb_top: f32) {
        if let Some(bar) = self.list_bar() {
            self.scroll_list_to(bar.first_at(thumb_top));
        }
    }

    pub(in crate::app::running) fn list_under_pointer(&self) -> Option<ScrollBar> {
        let point = self.pointer.position?;
        let list = self.frame.host_list()?;
        list.bar.filter(|_| list.rect.contains(point))
    }

    pub(in crate::app::running) fn scroll_list_to(&mut self, first: usize) {
        let Some(shown) = self.list_first_mut() else {
            return;
        };
        if *shown != first {
            *shown = first;
            self.changed();
        }
    }

    fn list_bar(&self) -> Option<ScrollBar> {
        self.frame.host_list()?.bar
    }
}
