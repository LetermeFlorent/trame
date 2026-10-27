use trame::preferences::{
    DEFAULT_HISTORY_LINES, MAX_HISTORY_LINES, MIN_HISTORY_LINES, Preferences,
};

#[test]
fn kept_lines_start_at_a_thousand_and_take_any_count_from_100_to_10_000() {
    let base = Preferences::default();
    assert!(base.efficiency);
    assert_eq!(base.history_lines, DEFAULT_HISTORY_LINES);
    assert_eq!(base.step_history_lines(-1).history_lines, 500);
    assert_eq!(base.step_history_lines(2).history_lines, 5_000);
    assert_eq!(
        base.step_history_lines(-100).history_lines,
        MIN_HISTORY_LINES
    );
    assert_eq!(
        base.step_history_lines(100).history_lines,
        MAX_HISTORY_LINES
    );
    let typed = base.typed_history_lines(" 3500 ");
    assert_eq!(typed.history_lines, 3_500);
    assert_eq!(typed.step_history_lines(1).history_lines, 5_000);
    assert_eq!(typed.step_history_lines(-1).history_lines, 2_000);
    for (typed, kept) in [
        ("42", MIN_HISTORY_LINES),
        ("250", 250),
        ("99999999999999999999999", MAX_HISTORY_LINES),
        ("", DEFAULT_HISTORY_LINES),
        ("lots", DEFAULT_HISTORY_LINES),
    ] {
        assert_eq!(
            base.typed_history_lines(typed).history_lines,
            kept,
            "{typed}"
        );
    }
}

#[test]
fn saved_kept_lines_outside_100_to_10_000_come_back_inside() {
    let base = Preferences::default();
    for (stored, kept) in [
        (0, MIN_HISTORY_LINES),
        (3_000, 3_000),
        (20_000, MAX_HISTORY_LINES),
        (usize::MAX, MAX_HISTORY_LINES),
    ] {
        let preferences = Preferences {
            history_lines: stored,
            ..base.clone()
        };
        assert_eq!(preferences.sanitized().history_lines, kept, "{stored}");
    }
}
