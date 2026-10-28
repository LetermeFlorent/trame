use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::text::font::discovery::{Finder, FontSource, SystemFonts};
use crate::text::font::{FontLibrary, system_fonts};

fn with_finder(finder: Finder) -> FontLibrary {
    FontLibrary::with_finder(&SystemFonts::default(), Some(finder))
}

fn counted(calls: &Arc<AtomicUsize>, answer: Option<FontSource>) -> Finder {
    let calls = Arc::clone(calls);
    Box::new(move |_, _| {
        calls.fetch_add(1, Ordering::SeqCst);
        answer.clone()
    })
}

fn monospace() -> FontSource {
    let mut terminal = system_fonts().terminal.into_iter();
    terminal.next().expect("a monospace font is installed")
}

fn settle(library: &mut FontLibrary) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut found = false;
    while library.fallbacks.lookups.is_busy() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(1));
        found |= library.receive_fallbacks();
    }
    found
}

#[test]
fn a_slow_font_lookup_does_not_hold_the_frame() {
    let mut library = with_finder(Box::new(|_, _| {
        thread::sleep(Duration::from_millis(1500));
        None
    }));
    let started = Instant::now();
    assert_eq!(library.fallback_for('x'), None);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn a_missing_character_is_looked_up_once() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut library = with_finder(counted(&calls, None));
    for _ in 0..3 {
        assert_eq!(library.fallback_for('x'), None);
    }
    assert!(!settle(&mut library));
    assert_eq!(library.fallback_for('x'), None);
    assert!(!library.fallbacks.lookups.is_busy());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn a_found_font_serves_the_character_once_its_answer_arrives() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut library = with_finder(counted(&calls, Some(monospace())));
    assert_eq!(library.fallback_for('x'), None);
    assert!(settle(&mut library));
    assert!(library.fallback_for('x').is_some());
}

#[test]
fn characters_queued_behind_a_lookup_use_the_font_it_found() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut library = with_finder(counted(&calls, Some(monospace())));
    assert_eq!(library.fallback_for('x'), None);
    assert_eq!(library.fallback_for('y'), None);
    assert!(settle(&mut library));
    assert!(library.fallback_for('y').is_some());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
