use std::collections::HashMap;

use crate::session::SessionId;
use crate::workspace::{InstanceId, MovingInstance, TabMark};

use super::{Desk, Effect, Effects, Leaving, Outcome};

pub struct TornTab {
    leaving: Leaving,
    mark: TabMark,
}

pub struct MovedInstance {
    moving: MovingInstance,
    sessions: Vec<(SessionId, Leaving)>,
}

impl Desk {
    pub fn can_tear(&self, tab: SessionId) -> bool {
        self.workspace.locate(tab).is_some() && !self.workspace.is_only_tab(tab)
    }

    pub fn tear_tab(&mut self, tab: SessionId) -> Outcome<Option<TornTab>> {
        let Some(mark) = self.workspace.take_tab(tab) else {
            return Outcome {
                value: None,
                effects: Effects::default(),
            };
        };
        let leaving = self.sessions.hand_off(tab);
        Outcome {
            value: Some(TornTab { leaving, mark }),
            effects: self.after_close(),
        }
    }

    pub fn give_instance(&mut self, id: InstanceId) -> Outcome<Option<MovedInstance>> {
        let Some(moving) = self.workspace.take_instance(id) else {
            return Outcome {
                value: None,
                effects: Effects::default(),
            };
        };
        let sessions = moving
            .instance
            .tabs()
            .into_iter()
            .map(|tab| (tab, self.sessions.hand_off(tab)))
            .collect();
        Outcome {
            value: Some(MovedInstance { moving, sessions }),
            effects: self.after_close(),
        }
    }

    pub fn adopt_torn(&mut self, torn: TornTab) -> Effects {
        let id = self.sessions.take_in(torn.leaving);
        self.workspace.add_instance(None, id);
        self.workspace.mark_tab(id, torn.mark);
        Effects::from([Effect::Relayout, Effect::StartShown, Effect::Changed])
    }

    pub fn receive_tab(&mut self, torn: TornTab, instance: Option<usize>) -> SessionId {
        let id = self.sessions.take_in(torn.leaving);
        if let Some(index) = instance {
            self.workspace.active = index;
        }
        self.workspace.add_tab(id);
        self.workspace.mark_tab(id, torn.mark);
        id
    }

    pub fn receive_instance(&mut self, moved: MovedInstance, index: Option<usize>) -> Effects {
        let renamed: HashMap<SessionId, SessionId> = moved
            .sessions
            .into_iter()
            .map(|(old, leaving)| (old, self.sessions.take_in(leaving)))
            .collect();
        let index = index.unwrap_or(self.workspace.instances.len());
        self.workspace.adopt_instance(moved.moving, &renamed, index);
        self.welcome()
    }

    pub fn welcome(&self) -> Effects {
        Effects::from([
            Effect::Animate,
            Effect::Relayout,
            Effect::StartShown,
            Effect::Changed,
            Effect::Raise,
        ])
    }
}
