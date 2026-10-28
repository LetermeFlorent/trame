mod page;
mod place;

use crate::geometry::{Point, Rect};
use crate::preferences::{ConnectForm, SshHost};
use crate::ui::frame::FrameLayout;
use crate::ui::frame::sidebar::scroll::{BarPart, ScrollBar};

use super::SettingLine;
pub use page::ConnectionsPage;
pub(super) use page::connections_page;
pub(in crate::ui::frame) use place::ListSpec;

pub(in crate::ui::frame) fn host_labels(hosts: &[SshHost]) -> Vec<String> {
    hosts.iter().map(SshHost::label).collect()
}

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

#[derive(Clone, Debug)]
pub struct HostList {
    pub rect: Rect,
    pub rows: Vec<HostRow>,
    pub bar: Option<ScrollBar>,
    pub pickable: bool,
}

impl HostList {
    pub fn control_at(&self, point: Point) -> Option<ListControl> {
        if let Some(part) = self.bar.and_then(|bar| bar.part_at(point)) {
            return Some(match part {
                BarPart::Thumb => ListControl::Thumb,
                BarPart::Track => ListControl::Track,
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
        let editor = self.editor.as_ref().and_then(|editor| editor.list.as_ref());
        editor.or_else(|| {
            let settings = self.settings.as_ref()?;
            let shortcuts = settings.shortcuts().and_then(|page| page.list.as_ref());
            let connections = settings.connections().and_then(|page| page.list.as_ref());
            connections.or(shortcuts)
        })
    }
}
