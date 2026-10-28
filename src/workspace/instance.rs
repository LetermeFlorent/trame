use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;

use super::pane::Pane;
use super::tree::{Edge, Node, PaneId};
use crate::session::SessionId;
use crate::theme::{Emblem, Swatch};

const NUMBER_SLOT: &str = "{number}";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstanceId(pub u32);

#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    pub id: InstanceId,
    pub name: String,
    pub color: Option<Swatch>,
    pub emblem: Emblem,
    pub number: u32,
    pub folder: Option<PathBuf>,
    pub root: Node,
    pub panes: BTreeMap<PaneId, Pane>,
    pub active_pane: PaneId,
}

impl Instance {
    pub fn shown_name(&self, phrase: &str) -> Cow<'_, str> {
        match self.name.is_empty() {
            true => phrase.replace(NUMBER_SLOT, &self.number.to_string()).into(),
            false => Cow::Borrowed(&self.name),
        }
    }

    pub fn new(id: InstanceId, name: String, pane: PaneId, tab: SessionId) -> Self {
        Self {
            id,
            name,
            color: None,
            emblem: Emblem::default(),
            number: 1,
            folder: None,
            root: Node::Pane(pane),
            panes: BTreeMap::from([(pane, Pane::new(tab))]),
            active_pane: pane,
        }
    }

    pub fn active(&self) -> Option<&Pane> {
        self.panes.get(&self.active_pane)
    }

    pub fn active_mut(&mut self) -> Option<&mut Pane> {
        self.panes.get_mut(&self.active_pane)
    }

    pub fn active_tab(&self) -> Option<SessionId> {
        self.active().and_then(Pane::active_tab)
    }

    pub fn tabs(&self) -> Vec<SessionId> {
        self.root
            .panes()
            .iter()
            .filter_map(|pane| self.panes.get(pane))
            .flat_map(|pane| pane.tabs.iter().copied())
            .collect()
    }

    pub fn tab_count(&self) -> usize {
        self.panes.values().map(|pane| pane.tabs.len()).sum()
    }

    pub fn find(&self, tab: SessionId) -> Option<PaneId> {
        self.panes
            .iter()
            .find(|(_, pane)| pane.position(tab).is_some())
            .map(|(id, _)| *id)
    }

    pub fn is_empty(&self) -> bool {
        self.panes.is_empty()
    }

    pub fn focus(&mut self, pane: PaneId) {
        if self.panes.contains_key(&pane) {
            self.active_pane = pane;
        }
    }

    pub fn split(&mut self, target: PaneId, new: PaneId, tab: SessionId, edge: Edge) -> bool {
        if !self.root.split(target, new, edge) {
            return false;
        }
        self.panes.insert(new, Pane::new(tab));
        self.active_pane = new;
        true
    }

    pub(super) fn insert(&mut self, pane: PaneId, tab: SessionId, index: Option<usize>) -> bool {
        let Some(destination) = self.panes.get_mut(&pane) else {
            return false;
        };
        destination.insert(tab, index);
        self.focus(pane);
        true
    }

    pub(super) fn insert_active(&mut self, tab: SessionId) -> bool {
        let Some(pane) = self.active_mut() else {
            return false;
        };
        pane.insert(tab, None);
        true
    }

    pub fn remove_tab(&mut self, tab: SessionId) -> bool {
        let Some(pane_id) = self.find(tab) else {
            return false;
        };
        let Some(pane) = self.panes.get_mut(&pane_id) else {
            return false;
        };
        pane.remove(tab);
        if pane.tabs.is_empty() {
            self.remove_pane(pane_id);
        }
        true
    }

    fn remove_pane(&mut self, pane: PaneId) {
        let order = self.root.panes();
        self.panes.remove(&pane);
        self.root.remove(pane);
        if self.active_pane != pane {
            return;
        }
        let index = order.iter().position(|id| *id == pane).unwrap_or(0);
        let remaining: Vec<PaneId> = order.into_iter().filter(|id| *id != pane).collect();
        if let Some(next) = remaining.get(index.saturating_sub(1)).or(remaining.first()) {
            self.active_pane = *next;
        }
    }
}
