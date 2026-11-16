use crate::session::SessionId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Redraw,
    Relayout,
    Changed,
    Rescale,
    Animate,
    Raise,
    StartQueued,
    StartShown,
    Typed(SessionId),
}

#[must_use]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effects(Vec<Effect>);

impl Effects {
    pub fn push(&mut self, effect: Effect) {
        self.0.push(effect);
    }

    pub fn append(&mut self, mut other: Effects) {
        self.0.append(&mut other.0);
    }

    pub fn as_slice(&self) -> &[Effect] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Effect> for Effects {
    fn from(effect: Effect) -> Self {
        Self(vec![effect])
    }
}

impl<const N: usize> From<[Effect; N]> for Effects {
    fn from(effects: [Effect; N]) -> Self {
        Self(effects.into())
    }
}

impl IntoIterator for Effects {
    type Item = Effect;
    type IntoIter = std::vec::IntoIter<Effect>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[must_use]
pub struct Outcome<T> {
    pub value: T,
    pub effects: Effects,
}
