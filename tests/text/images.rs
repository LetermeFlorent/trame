use trame::text::ImageKey;

use crate::system;

#[test]
fn an_image_enters_the_atlas_once_for_each_size() {
    let mut text = system();
    let key = ImageKey { id: 7, size: 4 };
    let mut drawn = 0;
    let first = text.image(key, || {
        drawn += 1;
        vec![255; 64]
    });
    let again = text.image(key, || {
        drawn += 1;
        vec![0; 64]
    });
    assert_eq!(drawn, 1);
    assert!(first.is_some() && first == again);
    let region = first.expect("room in the atlas");
    assert_eq!((region.width, region.height), (4, 4));
    let bigger = text.image(ImageKey { size: 8, ..key }, || vec![255; 256]);
    assert_eq!(bigger.map(|region| region.width), Some(8));
    assert_ne!(bigger, first);
}
