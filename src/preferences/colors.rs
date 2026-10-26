use super::Preferences;
use crate::theme::Swatch;

pub const MAX_CUSTOM_COLORS: usize = 8;

impl Preferences {
    pub fn with_custom_color(&self, slot: Option<usize>, color: Swatch) -> Self {
        self.with(|next| {
            match slot.filter(|index| *index < next.custom_colors.len()) {
                Some(index) => next.custom_colors[index] = color,
                None => next.custom_colors.push(color),
            }
            next.custom_colors = sanitized_colors(std::mem::take(&mut next.custom_colors));
        })
    }

    pub fn without_custom_color(&self, index: usize) -> Self {
        self.with(|next| {
            if index < next.custom_colors.len() {
                next.custom_colors.remove(index);
            }
        })
    }
}

pub(super) fn sanitized_colors(colors: Vec<Swatch>) -> Vec<Swatch> {
    let mut kept: Vec<Swatch> = Vec::new();
    for color in colors.into_iter().filter(|color| color.is_custom()) {
        if !kept.contains(&color) {
            kept.push(color);
        }
    }
    kept.truncate(MAX_CUSTOM_COLORS);
    kept
}
