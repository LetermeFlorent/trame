use trame::term::graphics::ImageStore;

#[test]
fn reinserting_an_id_bumps_the_serial_and_replaces_bytes() {
    let mut store = ImageStore::default();
    let first = store.insert(1, 1, 1, vec![0; 4]).serial;
    let second = store.insert(1, 2, 1, vec![0; 8]).serial;
    assert!(second > first);
    let image = store.get(1).unwrap();
    assert_eq!((image.width, image.height, image.pixels.len()), (2, 1, 8));
    assert_eq!(store.bytes(), 8 + ImageStore::OVERHEAD);
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
    let each = 4 + ImageStore::OVERHEAD;
    store.evict_until(2 * each, &|id| id == 1);
    assert!(store.get(1).is_some());
    assert!(store.get(2).is_none() && store.get(3).is_none());
    assert!(store.get(4).is_some());
    store.evict_until(each, &|_| true);
    assert!(store.get(1).is_none());
    assert_eq!(store.bytes(), each);
}

#[test]
fn remove_and_clear_free_bytes() {
    let mut store = ImageStore::default();
    store.insert(1, 1, 1, vec![0; 4]);
    store.insert(2, 1, 1, vec![0; 4]);
    store.remove(1);
    assert_eq!(store.bytes(), 4 + ImageStore::OVERHEAD);
    store.clear();
    assert_eq!(store.bytes(), 0);
    assert!(store.get(2).is_none());
    assert_eq!(ImageStore::QUOTA, 320 << 20);
}

#[test]
fn assigned_ids_skip_ids_already_in_use() {
    let mut store = ImageStore::default();
    store.insert(1 << 31, 1, 1, vec![0; 4]);
    store.insert((1 << 31) + 1, 1, 1, vec![0; 4]);
    assert_eq!(store.assign(0), (1 << 31) + 2);
}

#[test]
fn many_tiny_images_are_evicted_in_one_pass() {
    let mut store = ImageStore::default();
    for id in 1..=100_000 {
        store.insert(id, 1, 1, vec![0; 4]);
    }
    store.evict_until(0, &|id| id % 2 == 0);
    assert!(store.is_empty());
    assert_eq!(store.bytes(), 0);
}
