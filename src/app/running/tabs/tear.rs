use trame::session::SessionId;
use trame::workspace::TabMark;

use crate::app::running::Running;
use crate::app::sessions::Leaving;

pub struct TornTab {
    leaving: Leaving,
    mark: TabMark,
}

impl Running {
    pub fn can_tear(&self, tab: SessionId) -> bool {
        self.workspace.locate(tab).is_some() && !self.workspace.is_only_tab(tab)
    }

    pub fn tear_tab(&mut self, tab: SessionId) -> Option<TornTab> {
        let mark = self.workspace.take_tab(tab)?;
        let leaving = self.sessions.hand_off(tab);
        self.after_close();
        Some(TornTab { leaving, mark })
    }

    pub(in crate::app::running) fn take_torn_tab(&mut self, torn: TornTab) {
        let id = self.sessions.take_in(torn.leaving);
        self.workspace.add_instance(None, id);
        self.workspace.mark_tab(id, torn.mark);
        self.relayout();
        self.start_shown();
        self.changed();
    }
}
