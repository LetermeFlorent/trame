use std::collections::{HashMap, VecDeque};
use std::ops::RangeInclusive;

use unicode_width::UnicodeWidthChar;

use super::discovery::{Finder, FontSource};
use super::{FontId, FontKey, FontLibrary};

const EMOJI_RANGES: [RangeInclusive<u32>; 2] = [0x2300..=0x2BFF, 0x1F000..=0x1FAFF];

#[derive(Clone, Copy)]
enum Kind {
    Text,
    Emoji,
}

#[derive(Default)]
struct FallbackList {
    loaded: Vec<FontId>,
    pending: VecDeque<FontSource>,
}

#[derive(Default)]
pub struct Fallbacks {
    text: FallbackList,
    emoji: FallbackList,
    coverage: HashMap<char, Option<FontId>>,
    finder: Option<Finder>,
}

impl Fallbacks {
    pub fn new(text: &[FontSource], emoji: &[FontSource], finder: Option<Finder>) -> Self {
        Self {
            text: FallbackList {
                loaded: Vec::new(),
                pending: text.iter().cloned().collect(),
            },
            emoji: FallbackList {
                loaded: Vec::new(),
                pending: emoji.iter().cloned().collect(),
            },
            coverage: HashMap::new(),
            finder,
        }
    }

    fn list(&self, kind: Kind) -> &FallbackList {
        match kind {
            Kind::Text => &self.text,
            Kind::Emoji => &self.emoji,
        }
    }

    fn list_mut(&mut self, kind: Kind) -> &mut FallbackList {
        match kind {
            Kind::Text => &mut self.text,
            Kind::Emoji => &mut self.emoji,
        }
    }
}

impl FontLibrary {
    pub fn covering_font(
        &mut self,
        primary: FontKey,
        ch: char,
        weight: u16,
        italic: bool,
    ) -> Option<FontKey> {
        if self.face(primary.id).covers(ch) {
            return Some(primary);
        }
        let id = self.fallback_for(ch)?;
        Some(FontKey::synthesize(self, id, weight, italic))
    }

    pub fn fallback_for(&mut self, ch: char) -> Option<FontId> {
        if let Some(found) = self.fallbacks.coverage.get(&ch) {
            return *found;
        }
        let found = self.search_fallback(ch);
        self.fallbacks.coverage.insert(ch, found);
        found
    }

    fn search_fallback(&mut self, ch: char) -> Option<FontId> {
        let emoji = prefers_emoji(ch);
        let order = if emoji {
            [Kind::Emoji, Kind::Text]
        } else {
            [Kind::Text, Kind::Emoji]
        };
        order
            .into_iter()
            .find_map(|kind| self.search_list(kind, ch))
            .or_else(|| self.search_system(ch, emoji))
    }

    fn search_list(&mut self, kind: Kind, ch: char) -> Option<FontId> {
        let loaded = &self.fallbacks.list(kind).loaded;
        if let Some(id) = loaded.iter().copied().find(|id| self.face(*id).covers(ch)) {
            return Some(id);
        }
        while let Some(source) = self.fallbacks.list_mut(kind).pending.pop_front() {
            let ids = self.load(&source);
            self.fallbacks.list_mut(kind).loaded.extend(&ids);
            if let Some(id) = ids.into_iter().find(|id| self.face(*id).covers(ch)) {
                return Some(id);
            }
        }
        None
    }

    fn search_system(&mut self, ch: char, emoji: bool) -> Option<FontId> {
        let source = self.fallbacks.finder.as_mut()?(ch, emoji)?;
        let id = self
            .load(&source)
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

fn prefers_emoji(ch: char) -> bool {
    let code = u32::from(ch);
    ch.width() == Some(2) && EMOJI_RANGES.iter().any(|range| range.contains(&code))
}
