pub(super) fn fit_widths(natural: &[f32], available: f32, minimum: f32) -> Vec<f32> {
    let total: f32 = natural.iter().sum();
    if total <= available || total <= 0.0 {
        return natural.to_vec();
    }
    let scale = available / total;
    natural
        .iter()
        .map(|width| (width * scale).floor().max(minimum))
        .collect()
}

pub(super) fn scroll_offset(widths: &[f32], active: usize, available: f32) -> f32 {
    let end: f32 = widths.iter().take(active + 1).sum();
    (end - available).max(0.0)
}
