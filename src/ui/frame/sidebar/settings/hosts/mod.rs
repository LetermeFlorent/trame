mod page;
mod place;

use crate::geometry::{Point, Rect};
use crate::preferences::{ConnectForm, SshHost};
use crate::ui::frame::FrameLayout;

use super::SettingLine;
pub use page::ConnectionsPage;
pub(super) use page::connections_page;
pub(in crate::ui::frame) use place::ListSpec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListControl {
    Pick(usize),
    Change(usize),
    Forget(usize),
    Thumb,
    Track,
}

#[derive(Clone, Copy, Debug)]
pub struct HostsView<'a> {
    pub hosts: &'a [SshHost],
    pub first: usize,
    pub form: Option<&'a ConnectForm>,
    pub changing: bool,
}

#[derive(Clone, Debug)]
pub struct HostRow {
    pub index: usize,
    pub line: SettingLine,
    pub change: Rect,
    pub forget: Option<Rect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollBar {
    pub track: Rect,
    pub thumb: Rect,
    pub first: usize,
    pub last_first: usize,
    pub shown: usize,
}

impl ScrollBar {
    pub fn scrolled(&self, rows: isize) -> usize {
        self.first.saturating_add_signed(rows).min(self.last_first)
    }

    pub fn first_at(&self, thumb_top: f32) -> usize {
        let travel = self.track.height - self.thumb.height;
        if travel <= 0.0 {
            return self.first;
        }
        let ratio = ((thumb_top - self.track.y) / travel).clamp(0.0, 1.0);
        (ratio * self.last_first as f32).round() as usize
    }

    pub fn paged(&self, y: f32) -> usize {
        match y < self.thumb.y {
            true => self.first.saturating_sub(self.shown),
            false => (self.first + self.shown).min(self.last_first),
        }
    }
}

#[derive(Clone, Debug)]
pub struct HostList {
    pub rect: Rect,
    pub rows: Vec<HostRow>,
    pub bar: Option<ScrollBar>,
    pub pickable: bool,
}

impl HostList {
    pub fn control_at(&self, point: Point) -> Option<ListControl> {
        if let Some(bar) = self.bar.filter(|bar| bar.track.contains(point)) {
            return Some(match bar.thumb.contains(point) {
                true => ListControl::Thumb,
                false => ListControl::Track,
            });
        }
        let row = self.rows.iter().find(|row| row.line.rect.contains(point))?;
        match () {
            _ if row.change.contains(point) => Some(ListControl::Change(row.index)),
            _ if row.forget.is_some_and(|rect| rect.contains(point)) => {
                Some(ListControl::Forget(row.index))
            }
            _ => self.pickable.then_some(ListControl::Pick(row.index)),
        }
    }
}

impl FrameLayout {
    pub fn host_list(&self) -> Option<&HostList> {
        let settings = self.settings.as_ref().and_then(|settings| {
            let page = settings.connections()?;
            page.list.as_ref()
        });
        settings.or_else(|| self.editor.as_ref()?.list.as_ref())
    }
}
