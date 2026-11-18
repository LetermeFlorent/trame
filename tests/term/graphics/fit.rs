use trame::term::graphics::uv_to_source_rect;

const DST: (f32, f32, f32, f32) = (10.0, 20.0, 100.0, 50.0);

#[test]
fn a_full_uv_keeps_the_destination() {
    assert_eq!(uv_to_source_rect(DST, [0.0, 0.0, 1.0, 1.0]), Some(DST));
}

#[test]
fn a_half_uv_doubles_the_size() {
    assert_eq!(
        uv_to_source_rect(DST, [0.0, 0.0, 0.5, 0.5]),
        Some((10.0, 20.0, 200.0, 100.0))
    );
}

#[test]
fn an_offset_uv_shifts_the_image_back() {
    assert_eq!(
        uv_to_source_rect(DST, [0.5, 0.5, 1.0, 1.0]),
        Some((-90.0, -30.0, 200.0, 100.0))
    );
}

#[test]
fn an_empty_uv_has_no_source() {
    assert_eq!(uv_to_source_rect(DST, [0.3, 0.0, 0.3, 1.0]), None);
    assert_eq!(uv_to_source_rect(DST, [0.0, 0.6, 1.0, 0.2]), None);
}
