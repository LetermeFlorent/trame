use std::sync::Arc;

use crate::text::{TextLayout, TextStyle, TextSystem};

const ELLIPSIS: char = '\u{2026}';

#[derive(Clone, Copy)]
enum Cut {
    End,
    Start,
}

pub fn fit_label(
    text: &mut TextSystem,
    label: &str,
    style: TextStyle,
    max_width: f32,
) -> Arc<TextLayout> {
    let fitted = fit_text(text, label, style, max_width, Cut::End);
    text.layout(&fitted, style)
}

pub fn fit_label_start(
    text: &mut TextSystem,
    label: &str,
    style: TextStyle,
    max_width: f32,
) -> Arc<TextLayout> {
    let fitted = fit_text_start(text, label, style, max_width);
    text.layout(&fitted, style)
}

pub fn fit_text_start(
    text: &mut TextSystem,
    label: &str,
    style: TextStyle,
    max_width: f32,
) -> String {
    fit_text(text, label, style, max_width, Cut::Start)
}

fn fit_text(
    text: &mut TextSystem,
    label: &str,
    style: TextStyle,
    max_width: f32,
    cut: Cut,
) -> String {
    if label.is_empty() || text.layout(label, style).width <= max_width {
        return label.to_owned();
    }
    let bounds: Vec<usize> = label.char_indices().map(|(index, _)| index).collect();
    let mut low = 0;
    let mut high = bounds.len() - 1;
    while low < high {
        let middle = (low + high).div_ceil(2);
        if text.layout(&cut.keep(label, &bounds, middle), style).width <= max_width {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    cut.keep(label, &bounds, low)
}

impl Cut {
    fn keep(self, label: &str, bounds: &[usize], count: usize) -> String {
        match self {
            Self::End => format!("{}{ELLIPSIS}", label[..bounds[count]].trim_end()),
            Self::Start => {
                let start = bounds.get(bounds.len() - count).copied();
                let rest = &label[start.unwrap_or(label.len())..];
                format!("{ELLIPSIS}{}", rest.trim_start())
            }
        }
    }
}
