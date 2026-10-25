use serde::{Deserialize, Serialize};

use super::{Instance, Workspace};
use crate::theme::{Emblem, Letters};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Numbering {
    #[serde(rename = "instance_numbers")]
    pub shown: bool,
    #[serde(rename = "renumber_on_close")]
    pub on_close: bool,
    #[serde(rename = "renumber_on_move")]
    pub on_move: bool,
}

impl Default for Numbering {
    fn default() -> Self {
        Self {
            shown: true,
            on_close: true,
            on_move: false,
        }
    }
}

impl Workspace {
    pub fn numbered(numbering: Numbering) -> Self {
        Self {
            numbering,
            ..Self::new()
        }
    }

    pub fn set_numbering(&mut self, numbering: Numbering) {
        self.numbering = numbering;
    }

    pub fn shown_emblem(&self, instance: &Instance) -> Emblem {
        let number = Letters::parse(&instance.number.to_string());
        match (self.numbering.shown, number) {
            (true, Some(letters)) => Emblem::Letters(letters),
            _ => instance.emblem,
        }
    }

    pub(super) fn next_number(&self) -> u32 {
        let highest = self.instances.iter().map(|instance| instance.number).max();
        highest.unwrap_or(0) + 1
    }

    pub(super) fn renumber_after_close(&mut self) {
        if !self.numbering.on_close {
            return;
        }
        let mut order: Vec<usize> = (0..self.instances.len()).collect();
        order.sort_by_key(|index| self.instances[*index].number);
        for (rank, index) in order.into_iter().enumerate() {
            self.instances[index].number = rank as u32 + 1;
        }
    }

    pub fn move_instance(&mut self, from: usize, to: usize) -> bool {
        let count = self.instances.len();
        if from == to || from >= count || to >= count {
            return false;
        }
        let active = self.instances.get(self.active).map(|instance| instance.id);
        let moved = self.instances.remove(from);
        self.instances.insert(to, moved);
        self.active = active
            .and_then(|id| self.instance_index(id))
            .unwrap_or(self.active);
        true
    }

    pub fn renumber_after_move(&mut self) {
        if !self.numbering.on_move {
            return;
        }
        for (index, instance) in self.instances.iter_mut().enumerate() {
            instance.number = index as u32 + 1;
        }
    }
}
