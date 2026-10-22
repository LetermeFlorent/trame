use trame::preferences::{MAX_INTERFACE_SIZE, MIN_INTERFACE_SIZE, Preferences};
use trame::ui::metrics::{MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE, TERMINAL_FONT_SIZE};
use trame::workspace::{NodeSnapshot, PaneSnapshot, Snapshot, TabSnapshot};

use super::{restore, single};

const OLDER_FILE: &str = r#"{"version":1,"sidebar_open":true,"font_size":16.0,"active_instance":0,"instances":[{"name":"a","active_pane":0,"root":{"Pane":{"tabs":[{}],"active":0}}}]}"#;

fn with_interface(interface_size: u16) -> Preferences {
    Preferences {
        interface_size,
        ..Preferences::default()
    }
}

fn close(left: f32, right: f32) -> bool {
    (left - right).abs() < 1e-4
}

#[test]
fn older_files_get_the_default_preferences() {
    let snapshot = Snapshot::from_json(OLDER_FILE).expect("parses");
    assert_eq!(
        snapshot.preferences,
        Preferences::default().with_font_size(16.0)
    );
    assert!(snapshot.preferences.restore_text);
}

#[test]
fn font_size_stays_finite_and_in_range() {
    let base = Preferences::default();
    assert_eq!(base.with_font_size(100.0).font_size, MAX_TERMINAL_FONT_SIZE);
    assert_eq!(base.with_font_size(1.0).font_size, MIN_TERMINAL_FONT_SIZE);
    for odd in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(base.with_font_size(odd).font_size, TERMINAL_FONT_SIZE);
    }
    assert_eq!(base.step_font_size(2).font_size, TERMINAL_FONT_SIZE + 2.0);
    let stored = Preferences {
        font_size: f32::NAN,
        ..base
    };
    assert_eq!(stored.sanitized(), base);
}

#[test]
fn interface_size_rounds_to_its_step_and_stays_in_range() {
    for (stored, kept) in [
        (104, 100),
        (105, 110),
        (0, MIN_INTERFACE_SIZE),
        (999, MAX_INTERFACE_SIZE),
        (u16::MAX, MAX_INTERFACE_SIZE),
    ] {
        assert_eq!(with_interface(stored).sanitized().interface_size, kept);
    }
    let base = Preferences::default();
    assert_eq!(base.step_interface_size(1).interface_size, 110);
    assert_eq!(
        base.step_interface_size(-100).interface_size,
        MIN_INTERFACE_SIZE
    );
    let largest = with_interface(MAX_INTERFACE_SIZE);
    assert_eq!(largest.step_interface_size(1), largest);
}

#[test]
fn restore_toggle_flips_only_itself() {
    let base = Preferences::default();
    let off = base.toggle_restore_text();
    assert!(!off.restore_text);
    assert_eq!(off.toggle_restore_text(), base);
}

#[test]
fn preferences_and_text_keys_survive_json() {
    let tab = TabSnapshot {
        transcript: Some("abc-1".to_owned()),
        ..TabSnapshot::default()
    };
    let mut snapshot = single(NodeSnapshot::Pane(PaneSnapshot {
        tabs: vec![tab],
        active: 0,
    }));
    snapshot.preferences = Preferences {
        font_size: 18.0,
        restore_text: false,
        interface_size: 120,
    };
    let text = snapshot.to_json().expect("serializes");
    let parsed = Snapshot::from_json(&text).expect("parses");
    assert_eq!(parsed, snapshot);
    let (_, seen) = restore(&parsed);
    assert_eq!(seen[0].transcript.as_deref(), Some("abc-1"));
}

#[test]
fn interface_size_scales_the_chrome_but_not_the_terminal_font() {
    let preferences = with_interface(120);
    assert!(close(preferences.ui_scale(1.5), 1.8));
    assert!(close(
        preferences.font_pixels(1.5),
        TERMINAL_FONT_SIZE * 1.5
    ));
}
