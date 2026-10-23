use std::fs;

use trame::files::{ICON_SIZE, IconStore, import_icon, load_icon};

use crate::{Sandbox, solid};

const ICON_BYTES: usize = (ICON_SIZE * ICON_SIZE * 4) as usize;

#[test]
fn an_imported_picture_is_saved_once_at_icon_size() {
    let sandbox = Sandbox::new("import");
    let source = sandbox.picture("logo.png", 3, 5, &solid(3, 5, [10, 200, 30, 255]));
    let id = import_icon(&source, &sandbox.icons()).expect("the picture is imported");
    let saved = sandbox.icons().join(format!("{id:016x}.png"));
    assert!(saved.is_file());
    let pixels = load_icon(&sandbox.icons(), id).expect("the icon is read back");
    assert_eq!(pixels.len(), ICON_BYTES);
    let middle = ((ICON_SIZE / 2 * ICON_SIZE + ICON_SIZE / 2) * 4) as usize;
    assert_eq!(pixels[middle..middle + 4], [10, 200, 30, 255]);
    let copy = sandbox.picture("copy.png", 3, 5, &solid(3, 5, [10, 200, 30, 255]));
    assert_eq!(import_icon(&copy, &sandbox.icons()).ok(), Some(id));
    let other = sandbox.picture("other.png", 3, 5, &solid(3, 5, [0, 0, 0, 255]));
    assert_ne!(import_icon(&other, &sandbox.icons()).ok(), Some(id));
    let names = fs::read_dir(sandbox.icons())
        .expect("the icon folder")
        .count();
    assert_eq!(names, 2);
}

#[test]
fn a_file_that_is_not_a_png_is_refused() {
    let sandbox = Sandbox::new("refused");
    let source = sandbox.root.join("notes.png");
    fs::write(&source, "not a picture").expect("the file is written");
    assert!(import_icon(&source, &sandbox.icons()).is_err());
    assert!(!sandbox.icons().exists());
}

#[test]
fn the_store_reads_an_icon_only_when_asked() {
    let sandbox = Sandbox::new("store");
    let source = sandbox.picture("logo.png", 2, 2, &solid(2, 2, [255; 4]));
    let id = import_icon(&source, &sandbox.icons()).expect("the picture is imported");
    let mut store = IconStore::new(Some(sandbox.icons()));
    assert_eq!(store.dir(), Some(sandbox.icons().as_path()));
    assert!(store.pixels(id).is_none());
    store.load(id);
    assert_eq!(store.pixels(id).map(<[u8]>::len), Some(ICON_BYTES));
    store.load(id ^ 1);
    assert!(store.pixels(id ^ 1).is_none());
    fs::remove_dir_all(sandbox.icons()).expect("the icons are removed");
    store.load(id);
    assert!(store.pixels(id).is_some());
    store.reload(id);
    assert!(store.pixels(id).is_none());
}

#[test]
fn a_store_without_a_folder_has_no_icons() {
    let mut store = IconStore::default();
    store.load(7);
    assert!(store.dir().is_none() && store.pixels(7).is_none());
}
