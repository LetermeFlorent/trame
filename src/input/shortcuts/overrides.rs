use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::actions::{action_id, action_of};
use super::combo::Combo;
use super::defaults::default_keys;
use crate::ui::Platform;

pub const MAX_KEYS: usize = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Value", into = "BTreeMap<String, Vec<String>>")]
pub struct Overrides {
    keys: BTreeMap<usize, Vec<Combo>>,
}

impl Overrides {
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn keys(&self, action: usize) -> Option<&[Combo]> {
        self.keys.get(&action).map(Vec::as_slice)
    }

    pub fn entries(&self) -> impl Iterator<Item = (usize, &[Combo])> {
        self.keys
            .iter()
            .map(|(action, keys)| (*action, keys.as_slice()))
    }

    pub fn with_keys(&self, action: usize, keys: Vec<Combo>, platform: Platform) -> Self {
        let mut kept: Vec<Combo> = Vec::new();
        for combo in keys {
            if !kept.contains(&combo) && kept.len() < MAX_KEYS {
                kept.push(combo);
            }
        }
        let mut next = self.clone();
        match default_keys(platform).get(action) {
            Some(defaults) if *defaults == kept => next.keys.remove(&action),
            Some(_) => next.keys.insert(action, kept),
            None => None,
        };
        next
    }

    pub fn reset(&self, action: usize) -> Self {
        let mut next = self.clone();
        next.keys.remove(&action);
        next
    }
}

impl From<Value> for Overrides {
    fn from(value: Value) -> Self {
        let Value::Object(map) = value else {
            return Self::default();
        };
        let mut entries: Vec<(usize, &Value)> = map
            .iter()
            .filter_map(|(id, keys)| Some((action_of(id)?, keys)))
            .collect();
        entries.sort_by_key(|(action, _)| *action);
        let mut taken: Vec<Combo> = Vec::new();
        let mut keys = BTreeMap::new();
        for (action, list) in entries {
            let Some(list) = list.as_array() else {
                continue;
            };
            let parsed = list
                .iter()
                .filter_map(Value::as_str)
                .filter_map(Combo::parse);
            let mut kept = Vec::new();
            for combo in parsed {
                if !taken.contains(&combo) && kept.len() < MAX_KEYS {
                    taken.push(combo);
                    kept.push(combo);
                }
            }
            if kept.is_empty() && !list.is_empty() {
                continue;
            }
            keys.insert(action, kept);
        }
        Self { keys }
    }
}

impl From<Overrides> for BTreeMap<String, Vec<String>> {
    fn from(overrides: Overrides) -> Self {
        overrides
            .keys
            .iter()
            .map(|(action, keys)| {
                let texts = keys.iter().map(Combo::text).collect();
                (action_id(*action).to_string(), texts)
            })
            .collect()
    }
}
