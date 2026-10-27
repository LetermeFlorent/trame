use std::time::{Duration, Instant};

use super::view::Loader;

fn after(start: Instant, millis: u64) -> Instant {
    start + Duration::from_millis(millis)
}

fn shown(message: &str, percent: Option<u8>) -> (Loader, Instant) {
    let start = Instant::now();
    let mut loader = Loader::new(start);
    loader.show(message.to_owned(), percent, start);
    (loader, start)
}

#[test]
fn the_content_fades_in_while_it_rises_into_place() {
    let (loader, start) = shown("Checking", None);
    let first = loader.view(start);
    assert_eq!((first.opacity, first.content), (1.0, 0.0));
    assert!(first.shift > 0.0 && first.previous.is_none());
    let middle = loader.view(after(start, 160));
    assert!(middle.content > 0.0 && middle.content < 1.0);
    assert!(middle.shift > 0.0 && middle.shift < first.shift);
    let settled = loader.view(after(start, 400));
    assert_eq!((settled.content, settled.shift), (1.0, 0.0));
    assert!(loader.is_moving(start) && !loader.is_moving(after(start, 400)));
}

#[test]
fn a_new_message_fades_over_the_old_one() {
    let (mut loader, start) = shown("Checking", None);
    let changed = after(start, 500);
    loader.show("Downloading".to_owned(), Some(0), changed);
    let view = loader.view(changed);
    assert_eq!(view.message, "Downloading");
    assert_eq!((view.previous, view.blend), (Some("Checking"), 0.0));
    let middle = loader.view(after(changed, 110)).blend;
    assert!(middle > 0.0 && middle < 1.0);
    assert_eq!(loader.view(after(changed, 300)).blend, 1.0);
    loader.show("Downloading".to_owned(), Some(10), after(changed, 300));
    assert_eq!(loader.view(after(changed, 300)).blend, 1.0);
}

#[test]
fn the_bar_glides_to_each_new_share() {
    let (mut loader, start) = shown("Downloading", Some(0));
    let fraction = |loader: &Loader, at| loader.view(at).progress.expect("a bar").fraction;
    assert_eq!(fraction(&loader, start), 0.0);
    let moved = after(start, 500);
    loader.show("Downloading".to_owned(), Some(50), moved);
    assert_eq!(fraction(&loader, moved), 0.0);
    let middle = fraction(&loader, after(moved, 200));
    assert!(middle > 0.0 && middle < 0.5);
    loader.show("Downloading".to_owned(), Some(50), after(moved, 200));
    assert_eq!(fraction(&loader, after(moved, 200)), middle);
    assert_eq!(fraction(&loader, after(moved, 450)), 0.5);
    let label = loader.view(moved).progress.expect("a bar").label;
    assert_eq!(label, "50%");
}

#[test]
fn closing_fades_everything_out_with_a_slight_drop() {
    let (mut loader, start) = shown("Installing", Some(100));
    let closing = after(start, 1000);
    loader.close(closing);
    let first = loader.view(closing);
    assert_eq!((first.opacity, first.shift), (1.0, 0.0));
    let middle = loader.view(after(closing, 175));
    assert!(middle.opacity > 0.0 && middle.opacity < 1.0);
    assert!(middle.shift > 0.0);
    assert!(loader.is_moving(after(closing, 175)));
    assert!(!loader.is_gone(after(closing, 300)));
    assert!(loader.is_gone(after(closing, 350)));
    assert_eq!(loader.view(after(closing, 400)).opacity, 0.0);
}
