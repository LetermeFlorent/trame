use trame::theme::Hsv;

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
