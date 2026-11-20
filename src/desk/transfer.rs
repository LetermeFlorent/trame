use std::collections::HashMap;

use crate::session::SessionId;
use crate::workspace::{InstanceId, MovingInstance, TabMark};

use super::{Desk, Effect, Effects, Leaving, Outcome};

pub struct TornTab {
    leaving: Leaving,
    mark: TabMark,
    transcript: Option<String>,
}

pub struct MovedInstance {
    moving: MovingInstance,
    sessions: Vec<(SessionId, Leaving, Option<String>)>,
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
        let transcript = self.transcripts.key(tab).map(str::to_owned);
        Outcome {
            value: Some(TornTab {
                leaving,
                mark,
                transcript,
            }),
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
            .map(|tab| {
                let transcript = self.transcripts.key(tab).map(str::to_owned);
                (tab, self.sessions.hand_off(tab), transcript)
            })
            .collect();
        Outcome {
            value: Some(MovedInstance { moving, sessions }),
            effects: self.after_close(),
        }
    }

    pub fn adopt_torn(&mut self, torn: TornTab) -> Effects {
        let id = self.take_in_tab(torn.leaving, torn.transcript.as_deref());
        self.workspace.add_instance(None, id);
        self.workspace.mark_tab(id, torn.mark);
        Effects::from([Effect::Relayout, Effect::StartShown, Effect::Changed])
    }

    pub fn receive_tab(&mut self, torn: TornTab, instance: Option<usize>) -> SessionId {
        let id = self.take_in_tab(torn.leaving, torn.transcript.as_deref());
        if let Some(index) = instance {
            self.workspace.active = index;
        }
        self.workspace.add_tab(id);
        self.workspace.mark_tab(id, torn.mark);
        id
    }

    pub fn receive_instance(&mut self, moved: MovedInstance, index: Option<usize>) -> Effects {
        let mut renamed = HashMap::new();
        for (old, leaving, transcript) in moved.sessions {
            let new = self.take_in_tab(leaving, transcript.as_deref());
            renamed.insert(old, new);
        }
        let index = index.unwrap_or(self.workspace.instances.len());
        self.workspace.adopt_instance(moved.moving, &renamed, index);
        self.welcome()
    }

    fn take_in_tab(&mut self, leaving: Leaving, transcript: Option<&str>) -> SessionId {
        let id = self.sessions.take_in(leaving);
        self.transcripts.adopt(id, transcript);
        id
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
