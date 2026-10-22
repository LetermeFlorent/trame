use std::time::Instant;

use super::tween::Tween;
use crate::ui::hit::Hit;

#[derive(Debug, Default)]
pub struct Fades {
    current: Option<Hit>,
    tweens: Vec<(Hit, Tween<f32>)>,
}

impl Fades {
    pub fn hover(&mut self, hit: Option<Hit>, now: Instant) {
        if hit == self.current {
            return;
        }
        let lit: Vec<Hit> = hit.into_iter().flat_map(surfaces).collect();
        for (surface, tween) in &mut self.tweens {
            let target = if lit.contains(surface) { 1.0 } else { 0.0 };
            tween.retarget(target, now);
        }
        for surface in lit {
            if !self.tweens.iter().any(|(known, _)| *known == surface) {
                self.tweens.push((surface, Tween::between(0.0, 1.0, now)));
            }
        }
        self.tweens
            .retain(|(_, tween)| tween.target() > 0.0 || !tween.is_done(now));
        self.current = hit;
    }

    pub fn glow(&self, hit: Hit, now: Instant) -> f32 {
        self.tweens
            .iter()
            .find(|(known, _)| *known == hit)
            .map_or(0.0, |(_, tween)| tween.value(now))
    }

    pub fn is_busy(&self, now: Instant) -> bool {
        self.tweens.iter().any(|(_, tween)| !tween.is_done(now))
    }
}

pub fn surfaces(hit: Hit) -> Vec<Hit> {
    match hit {
        Hit::RowClose(item) => vec![hit, Hit::Row(item)],
        Hit::TabClose(pane, tab) => vec![hit, Hit::Tab(pane, tab)],
        _ => vec![hit],
    }
}
