use crate::feed;

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
