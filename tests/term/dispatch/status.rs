use std::time::{Duration, Instant};

use crate::feed;

const WORKING: &str = "\x1b]7501;state=working:app=claude\x1b\\";

#[test]
fn the_probe_gets_the_supported_states_in_the_same_terminator() {
    let mut terminal = feed(1, 10, "\x1b]7501;?\x07");
    let reply = String::from_utf8(terminal.screen_mut().take_responses()).unwrap();
    assert_eq!(
        reply,
        "\x1b]7501;?:states=idle,working,done,blocked,error:kinds=permission,question,auth\x07"
    );
}

#[test]
fn a_working_program_stays_busy_until_it_reports_another_state() {
    let busy = feed(1, 10, WORKING);
    assert!(busy.screen().program_working());
    let idle = feed(1, 10, &format!("{WORKING}\x1b]7501;state=idle\x07"));
    assert!(!idle.screen().program_working());
    let prompt = feed(1, 10, &format!("{WORKING}\x1b]133;A\x07"));
    assert!(!prompt.screen().program_working());
    let cleared = feed(1, 10, &format!("{WORKING}\x1b]7501;state=clear\x07"));
    assert!(!cleared.screen().program_working());
}

#[test]
fn a_silent_program_stops_looking_busy_until_it_prints_again() {
    for start in [WORKING, "\x1b]133;C\x07"] {
        let mut terminal = feed(1, 10, start);
        let later = Instant::now() + Duration::from_secs(16);
        assert!(terminal.screen().program_working());
        assert!(!terminal.screen().program_working_at(later), "{start}");
        terminal.feed(b"\r|");
        assert!(terminal.screen().program_working(), "{start}");
    }
}

#[test]
fn an_ai_showing_its_idle_mark_stops_looking_busy_while_its_status_line_refreshes() {
    let mut terminal = feed(1, 10, "\x1b]133;C\x07\x1b]0;\u{25d0} Claude Code\x07");
    assert!(terminal.screen().program_working());
    terminal.feed("\x1b]0;\u{2733} Claude Code\x07\r cpu 41%".as_bytes());
    assert!(!terminal.screen().program_working());
    terminal.feed("\x1b]0;\u{25d1} Claude Code\x07".as_bytes());
    assert!(terminal.screen().program_working());
    terminal.feed("\x1b]0;\u{2733} Claude Code\x07\x1b]133;A\x07\x1b]133;C\x07".as_bytes());
    assert!(terminal.screen().program_working());
}

#[test]
fn the_idle_mark_is_forgotten_when_the_ai_exits() {
    for end in ["\x1b]133;D;0\x07", "\x1b]133;A\x07"] {
        let mark = "\x1b]133;C\x07\x1b]0;\u{2733} Claude Code\x07";
        let mut terminal = feed(1, 10, mark);
        assert!(terminal.screen().program_resting());
        terminal.feed(end.as_bytes());
        assert!(!terminal.screen().program_resting(), "{end}");
    }
}

#[test]
fn the_idle_mark_is_forgotten_when_the_shell_comes_back_to_the_front() {
    let mut terminal = feed(1, 10, "\x1b]0;\u{2733} Claude Code\x07");
    terminal.screen_mut().shell_in_front(false);
    assert!(terminal.screen().program_resting());
    terminal.screen_mut().shell_in_front(true);
    assert!(!terminal.screen().program_resting());
}

#[test]
fn the_idle_mark_stays_while_the_ai_stays_in_front() {
    let mut terminal = feed(1, 10, "\x1b]0;\u{2733} Claude Code\x07");
    for _ in 0..3 {
        terminal.screen_mut().shell_in_front(false);
        assert!(terminal.screen().program_resting());
    }
}

#[test]
fn the_idle_mark_survives_a_full_reset() {
    let terminal = feed(1, 10, "\x1b]0;\u{2733} Claude Code\x07\x1bc");
    assert!(terminal.screen().program_resting());
}

#[test]
fn the_idle_mark_stays_after_a_full_reset_when_the_shell_is_the_ai() {
    let title = "\x1b]0;\u{2733} Claude Code\x07";
    let mut terminal = feed(1, 10, title);
    terminal.screen_mut().shell_in_front(true);
    terminal.feed(title.as_bytes());
    terminal.feed(b"\x1bc");
    terminal.screen_mut().shell_in_front(true);
    assert!(terminal.screen().program_resting());
}

#[test]
fn a_running_command_is_busy_until_its_prompt_unless_its_program_reports() {
    let start = "\x1b]133;C\x07";
    assert!(feed(1, 10, start).screen().program_working());
    let ended = format!("{start}\x1b]133;D;0\x07");
    assert!(!feed(1, 10, &ended).screen().program_working());
    let prompt = format!("{start}\x1b]133;A\x07");
    assert!(!feed(1, 10, &prompt).screen().program_working());
    let reported = format!("{start}\x1b]7501;state=idle\x07");
    assert!(!feed(1, 10, &reported).screen().program_working());
    let stale = format!("\x1b]7501;state=done\x07{start}");
    assert!(feed(1, 10, &stale).screen().program_working());
}

#[test]
fn a_child_record_keeps_the_program_busy_until_its_parent_is_cleared() {
    let child = "\x1b]7501;state=working:id=build/test\x07";
    assert!(feed(1, 10, child).screen().program_working());
    let gone = feed(1, 10, &format!("{child}\x1b]7501;state=clear:id=build\x07"));
    assert!(!gone.screen().program_working());
}

#[test]
fn waiting_and_finished_programs_ask_for_attention_once() {
    let mut blocked = feed(
        1,
        10,
        "\x1b]7501;state=blocked:kind=permission:msg=UnVuIGxzPw==\x07",
    );
    assert_eq!(
        blocked.screen_mut().take_attention().as_deref(),
        Some("Run ls?")
    );
    blocked.feed(b"\x1b]7501;state=blocked:kind=permission\x07");
    assert_eq!(blocked.screen_mut().take_attention(), None);
    let mut done = feed(1, 10, "\x1b]7501;state=done\x07");
    assert_eq!(done.screen_mut().take_attention().as_deref(), Some(""));
    assert_eq!(feed(1, 10, WORKING).screen_mut().take_attention(), None);
}

#[test]
fn malformed_reports_change_nothing() {
    for report in [
        "state=working:id=bad id",
        "state=working:msg=@@@",
        "state=working:msg=YQpi",
        "state=running",
        "app=claude",
    ] {
        let terminal = feed(1, 10, &format!("\x1b]7501;{report}\x07"));
        assert!(!terminal.screen().program_working(), "{report}");
    }
}

#[test]
fn a_finished_command_reports_its_exit_code_once() {
    let mut ran = feed(1, 10, "]133;C]133;D;255]133;A");
    assert_eq!(ran.screen_mut().take_exit(), Some(255));
    assert_eq!(ran.screen_mut().take_exit(), None);
    let mut prompt = feed(1, 10, "]133;D;1]133;A");
    assert_eq!(prompt.screen_mut().take_exit(), None);
}
