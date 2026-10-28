use std::mem;

use super::Kind;
use super::lookups::Waker;
use crate::text::font::discovery::FontSource;
use crate::text::font::{FontId, FontLibrary};

impl FontLibrary {
    pub fn set_fallback_waker(&mut self, waker: Waker) {
        self.fallbacks.lookups.set_waker(waker);
    }

    pub fn receive_fallbacks(&mut self) -> bool {
        while let Some(answer) = self.fallbacks.lookups.answer() {
            let found = answer
                .source
                .and_then(|source| self.adopt(&source, answer.ch, answer.emoji));
            self.settle(answer.ch, found);
        }
        self.send_next();
        mem::take(&mut self.fallbacks.found)
    }

    pub(super) fn send_next(&mut self) {
        while let Some((ch, emoji)) = self.fallbacks.lookups.next() {
            if let Some(id) = self.search_lists(ch, emoji) {
                self.settle(ch, Some(id));
            } else if !self.fallbacks.lookups.send(ch, emoji) {
                self.settle(ch, None);
            }
        }
    }

    fn settle(&mut self, ch: char, found: Option<FontId>) {
        self.fallbacks.lookups.settle(ch);
        self.fallbacks.coverage.insert(ch, found);
        self.fallbacks.found |= found.is_some();
    }

    fn adopt(&mut self, source: &FontSource, ch: char, emoji: bool) -> Option<FontId> {
        let id = self
            .load(source)
            .into_iter()
            .find(|id| self.face(*id).covers(ch))?;
        let kind = if emoji { Kind::Emoji } else { Kind::Text };
        let list = self.fallbacks.list_mut(kind);
        if !list.loaded.contains(&id) {
            list.loaded.push(id);
        }
        Some(id)
    }
}
