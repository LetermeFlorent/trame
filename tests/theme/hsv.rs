use trame::theme::{Hsv, Swatch, hex_code, parse_hex};

#[test]
fn a_hex_code_reads_with_or_without_its_hash_and_in_short_form() {
    assert_eq!(parse_hex("#12ab34"), Some(0x12ab34));
    assert_eq!(parse_hex("12AB34"), Some(0x12ab34));
    assert_eq!(parse_hex("#fa0"), Some(0xffaa00));
    assert_eq!(parse_hex("#12ab3"), None);
    assert_eq!(parse_hex("#12ab3g"), None);
    assert_eq!(hex_code(0x12ab34), "#12AB34");
}

#[test]
fn every_colour_survives_the_trip_through_hue_saturation_and_value() {
    for rgb in [
        0x000000, 0xffffff, 0xff0000, 0x00ff00, 0x0000ff, 0x12ab34, 0x808080,
    ] {
        assert_eq!(Hsv::from_rgb(rgb).to_rgb(), rgb);
    }
    let red = Hsv::from_rgb(0x7f1010);
    assert_eq!(red.pure_hue(), 0xff0000);
}

#[test]
fn a_custom_colour_is_stored_as_its_hex_code_and_a_preset_by_its_name() {
    let custom = Swatch::custom(0x12ab34);
    assert_eq!(String::from(custom), "#12AB34");
    assert_eq!(String::from(Swatch::Teal), "teal");
    assert_eq!(Swatch::parse("teal"), Some(Swatch::Teal));
    assert_eq!(Swatch::parse("#12ab34"), Some(custom));
    assert_eq!(Swatch::parse("mauve"), None);
}
