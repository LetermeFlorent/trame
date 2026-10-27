use super::ShortcutInput;
use super::actions::ACTIONS;
use super::combo::Combo;
use super::defaults::default_keys;
use super::fit::Fit;
use super::overrides::Overrides;
use crate::input::Command;
use crate::ui::Platform;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keymap {
    bindings: Vec<(usize, Combo)>,
}

impl Keymap {
    pub fn new(platform: Platform, overrides: &Overrides) -> Self {
        let mut bindings: Vec<(usize, Combo)> = overrides
            .entries()
            .flat_map(|(action, keys)| keys.iter().map(move |combo| (action, *combo)))
            .collect();
        for (action, keys) in default_keys(platform).into_iter().enumerate() {
            if overrides.keys(action).is_some() {
                continue;
            }
            for combo in keys {
                if bindings.iter().all(|(_, taken)| *taken != combo) {
                    bindings.push((action, combo));
                }
            }
        }
        Self { bindings }
    }

    pub fn action_for(&self, input: ShortcutInput) -> Option<usize> {
        let first = |wanted: Fit| {
            self.bindings
                .iter()
                .find(|(_, combo)| combo.fit(&input) == Some(wanted))
                .map(|(action, _)| *action)
        };
        first(Fit::Exact).or_else(|| first(Fit::Loose))
    }

    pub fn command_for(&self, input: ShortcutInput) -> Option<Command> {
        self.action_for(input).map(|action| ACTIONS[action])
    }

    pub fn keys(&self, action: usize) -> Vec<Combo> {
        self.bindings
            .iter()
            .filter(|(owner, _)| *owner == action)
            .map(|(_, combo)| *combo)
            .collect()
    }

    pub fn owner(&self, combo: &Combo) -> Option<usize> {
        self.bindings
            .iter()
            .find(|(_, taken)| taken == combo)
            .map(|(action, _)| *action)
    }
}
