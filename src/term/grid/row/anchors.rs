use super::Row;
use crate::term::graphics::Anchor;

pub(super) type AnchorStore = Option<Box<[Anchor]>>;

const MAX_ANCHORS_PER_ROW: usize = 32;

impl Row {
    pub fn anchors(&self) -> &[Anchor] {
        self.anchors.as_deref().unwrap_or_default()
    }

    pub fn add_anchor(&mut self, anchor: Anchor) {
        let mut list = self.anchors.take().map(Vec::from).unwrap_or_default();
        list.retain(|found| !found.same_placement(&anchor));
        if list.len() >= MAX_ANCHORS_PER_ROW {
            list.drain(..=list.len() - MAX_ANCHORS_PER_ROW);
        }
        list.push(anchor);
        self.anchors = Some(list.into_boxed_slice());
    }

    pub fn retain_anchors(&mut self, keep: impl Fn(&Anchor) -> bool) {
        let Some(list) = self.anchors.take() else {
            return;
        };
        let list: Vec<_> = list.into_iter().filter(|anchor| keep(anchor)).collect();
        self.anchors = (!list.is_empty()).then(|| list.into_boxed_slice());
    }

    pub fn clear_anchors(&mut self) {
        self.anchors = None;
    }

    pub(in crate::term::grid) fn hand_anchors_down(&mut self, below: &mut Row) {
        let Some(list) = self.anchors.take() else {
            return;
        };
        for anchor in list.iter().filter_map(Anchor::without_top_row) {
            below.add_anchor(anchor);
        }
    }
}
