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
