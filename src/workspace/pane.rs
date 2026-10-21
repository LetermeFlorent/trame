use crate::session::SessionId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pane {
    pub tabs: Vec<SessionId>,
    pub active: usize,
}

impl Pane {
    pub fn new(tab: SessionId) -> Self {
        Self {
            tabs: vec![tab],
            active: 0,
        }
    }

    pub fn active_tab(&self) -> Option<SessionId> {
        self.tabs.get(self.active).copied()
    }

    pub fn position(&self, tab: SessionId) -> Option<usize> {
        self.tabs.iter().position(|candidate| *candidate == tab)
    }

    pub fn insert(&mut self, tab: SessionId, index: Option<usize>) {
        let index = index.unwrap_or(self.tabs.len()).min(self.tabs.len());
        self.tabs.insert(index, tab);
        self.active = index;
    }

    pub fn remove(&mut self, tab: SessionId) -> bool {
        let Some(index) = self.position(tab) else {
            return false;
        };
        self.tabs.remove(index);
        if index < self.active || self.active >= self.tabs.len() {
            self.active = self.active.saturating_sub(1);
        }
        true
    }

    pub fn activate(&mut self, tab: SessionId) -> bool {
        match self.position(tab) {
            Some(index) => {
                self.active = index;
                true
            }
            None => false,
        }
    }

    pub fn reorder(&mut self, tab: SessionId, index: usize) {
        let Some(from) = self.position(tab) else {
            return;
        };
        self.tabs.remove(from);
        let to = if index > from { index - 1 } else { index };
        let to = to.min(self.tabs.len());
        self.tabs.insert(to, tab);
        self.active = to;
    }

    pub fn cycle(&mut self, step: isize) {
        let count = self.tabs.len() as isize;
        if count > 0 {
            self.active = (self.active as isize + step).rem_euclid(count) as usize;
        }
    }
}
