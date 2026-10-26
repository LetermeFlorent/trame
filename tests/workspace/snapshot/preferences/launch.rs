use trame::input::MAX_CHARS;
use trame::preferences::{
    DEFAULT_HISTORY_LINES, HISTORY_CHOICES, MAX_INSTANCE_TABS, MIN_INSTANCE_TABS, Preferences,
    Recall,
};

fn with_command(command: &str) -> Preferences {
    Preferences::default().with_startup_command(command)
}

#[test]
fn everything_is_remembered_and_new_instances_open_one_terminal_by_default() {
    let base = Preferences::default();
    for recall in Recall::ALL {
        assert!(base.recalls(recall), "{recall:?}");
        assert!(base.can_recall(recall), "{recall:?}");
    }
    assert_eq!(base.instance_tabs, 1);
    assert_eq!(base.startup_command, None);
    assert_eq!(base.startup_line(), None);
    let saved = r#"{"font_size":14.0,"restore_text":true}"#;
    let older: Preferences = serde_json::from_str(saved).expect("parses");
    assert_eq!(older, base);
    let json = serde_json::to_string(&base).expect("serializes");
    assert!(!json.contains("startup_command"));
}

#[test]
fn each_switch_flips_only_itself() {
    let base = Preferences::default();
    let off = base.toggle(Recall::Instances);
    assert!(!off.restore_instances && off.restore_tabs && off.restore_text);
    assert_eq!(off.toggle(Recall::Instances), base);
    let tabs_off = base.toggle(Recall::Tabs);
    assert!(tabs_off.restore_instances && !tabs_off.restore_tabs && tabs_off.restore_text);
    let text_off = base.toggle(Recall::Text);
    assert!(text_off.restore_instances && text_off.restore_tabs && !text_off.restore_text);
}

#[test]
fn a_switch_turned_off_turns_off_the_ones_that_need_it() {
    let no_instances = Preferences::default().toggle(Recall::Instances);
    assert!(!no_instances.recalls(Recall::Tabs));
    assert!(!no_instances.recalls(Recall::Text));
    assert!(!no_instances.can_recall(Recall::Tabs));
    assert!(!no_instances.can_recall(Recall::Text));
    assert!(no_instances.can_recall(Recall::Instances));
    let no_tabs = Preferences::default().toggle(Recall::Tabs);
    assert!(no_tabs.recalls(Recall::Instances));
    assert!(!no_tabs.recalls(Recall::Text));
    assert!(no_tabs.can_recall(Recall::Tabs));
    assert!(!no_tabs.can_recall(Recall::Text));
}

#[test]
fn the_terminal_count_stays_between_one_and_eight() {
    let base = Preferences::default();
    assert_eq!(base.step_instance_tabs(-1), base);
    assert_eq!(base.step_instance_tabs(2).instance_tabs, 3);
    assert_eq!(
        base.step_instance_tabs(1000).instance_tabs,
        MAX_INSTANCE_TABS
    );
    for (stored, kept) in [
        (0, MIN_INSTANCE_TABS),
        (5, 5),
        (9, MAX_INSTANCE_TABS),
        (u8::MAX, MAX_INSTANCE_TABS),
    ] {
        let preferences = Preferences {
            instance_tabs: stored,
            ..base.clone()
        };
        assert_eq!(preferences.sanitized().instance_tabs, kept, "{stored}");
    }
}

#[test]
fn kept_lines_start_at_a_thousand_and_move_between_the_offered_choices() {
    let base = Preferences::default();
    assert!(base.efficiency);
    assert_eq!(base.history_lines, DEFAULT_HISTORY_LINES);
    assert_eq!(base.step_history_lines(-1), base);
    assert_eq!(base.step_history_lines(2).history_lines, 5_000);
    let last = HISTORY_CHOICES[HISTORY_CHOICES.len() - 1];
    assert_eq!(base.step_history_lines(100).history_lines, last);
    for (stored, kept) in [
        (0, 1_000),
        (3_000, 2_000),
        (10_000, 10_000),
        (usize::MAX, last),
    ] {
        let preferences = Preferences {
            history_lines: stored,
            ..base.clone()
        };
        assert_eq!(preferences.sanitized().history_lines, kept, "{stored}");
    }
}

#[test]
fn the_startup_command_is_trimmed_and_runs_with_enter() {
    let typed = with_command("  cd ~/work  ");
    assert_eq!(typed.startup_command.as_deref(), Some("cd ~/work"));
    assert_eq!(typed.startup_line().as_deref(), Some("cd ~/work\r"));
    for empty in ["", "   ", "\n\t"] {
        assert_eq!(with_command(empty).startup_command, None, "{empty:?}");
    }
    assert_eq!(
        with_command("ls\n; pwd").startup_command.as_deref(),
        Some("ls; pwd")
    );
    let long = "x".repeat(500);
    let kept = with_command(&long).startup_command.expect("kept");
    assert_eq!(kept.chars().count(), MAX_CHARS);
}

#[test]
fn a_stored_command_with_control_characters_is_cleaned_on_load() {
    let saved = r#"{"font_size":14.0,"startup_command":"\u001b[2Jclear\r"}"#;
    let loaded: Preferences = serde_json::from_str(saved).expect("parses");
    let cleaned = loaded.sanitized();
    assert_eq!(cleaned.startup_command.as_deref(), Some("[2Jclear"));
    let blank = r#"{"font_size":14.0,"startup_command":"  "}"#;
    let loaded: Preferences = serde_json::from_str(blank).expect("parses");
    assert_eq!(loaded.sanitized().startup_command, None);
}
