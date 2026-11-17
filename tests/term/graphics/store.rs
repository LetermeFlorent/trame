use trame::term::graphics::ImageStore;

#[test]
fn reinserting_an_id_bumps_the_serial_and_replaces_bytes() {
    let mut store = ImageStore::default();
    let first = store.insert(1, 1, 1, vec![0; 4]).serial;
    let second = store.insert(1, 2, 1, vec![0; 8]).serial;
    assert!(second > first);
    let image = store.get(1).unwrap();
    assert_eq!((image.width, image.height, image.pixels.len()), (2, 1, 8));
    assert_eq!(store.bytes(), 8);
}

#[test]
fn assigns_ids_from_the_high_range() {
    let mut store = ImageStore::default();
    let first = store.assign(7);
    let second = store.assign(0);
    assert_eq!(first, 1 << 31);
    assert_eq!(second, first + 1);
    assert_eq!(store.by_number(7), Some(first));
    store.remove(first);
    assert_eq!(store.by_number(7), None);
}

#[test]
fn evicts_oldest_unkept_images_first() {
    let mut store = ImageStore::default();
    for id in 1..=4 {
        store.insert(id, 1, 1, vec![0; 4]);
    }
    store.evict_until(8, &|id| id == 1);
    assert!(store.get(1).is_some());
    assert!(store.get(2).is_none() && store.get(3).is_none());
    assert!(store.get(4).is_some());
    store.evict_until(4, &|_| true);
    assert!(store.get(1).is_none());
    assert_eq!(store.bytes(), 4);
}

#[test]
fn remove_and_clear_free_bytes() {
    let mut store = ImageStore::default();
    store.insert(1, 1, 1, vec![0; 4]);
    store.insert(2, 1, 1, vec![0; 4]);
    store.remove(1);
    assert_eq!(store.bytes(), 4);
    store.clear();
    assert_eq!(store.bytes(), 0);
    assert!(store.get(2).is_none());
    assert_eq!(ImageStore::QUOTA, 320 << 20);
}
