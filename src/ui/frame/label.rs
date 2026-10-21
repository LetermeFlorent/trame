use std::sync::Arc;

use crate::text::{TextLayout, TextStyle, TextSystem};

const ELLIPSIS: char = '\u{2026}';

pub fn fit_label(
    text: &mut TextSystem,
    label: &str,
    style: TextStyle,
    max_width: f32,
) -> Arc<TextLayout> {
    let full = text.layout(label, style);
    if full.width <= max_width || label.is_empty() {
        return full;
    }
    let ends: Vec<usize> = label.char_indices().map(|(index, _)| index).collect();
    let mut low = 0;
    let mut high = ends.len() - 1;
    while low < high {
        let middle = (low + high).div_ceil(2);
        if shortened(text, label, ends[middle], style).width <= max_width {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    shortened(text, label, ends[low], style)
}

fn shortened(text: &mut TextSystem, label: &str, end: usize, style: TextStyle) -> Arc<TextLayout> {
    let candidate = format!("{}{ELLIPSIS}", label[..end].trim_end());
    text.layout(&candidate, style)
}
