pub fn uv_to_source_rect(dst: (f32, f32, f32, f32), uv: [f32; 4]) -> Option<(f32, f32, f32, f32)> {
    let (x, y, width, height) = dst;
    let [u0, v0, u1, v1] = uv;
    let (span_u, span_v) = (u1 - u0, v1 - v0);
    if span_u <= 0.0 || span_v <= 0.0 {
        return None;
    }
    let (full_width, full_height) = (width / span_u, height / span_v);
    Some((
        x - u0 * full_width,
        y - v0 * full_height,
        full_width,
        full_height,
    ))
}
