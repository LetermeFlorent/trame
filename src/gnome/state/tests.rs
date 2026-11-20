use std::sync::Arc;

use trame::desk::DeskSetup;
use trame::host::persist::Slot;
use trame::preferences::{Preferences, SshHost};
use trame::session::SessionId;
use trame::term::Screen;
use trame::theme::Appearance;
use trame::ui::frame::SettingControl;
use trame::ui::{Hit, PaneFrame};

use super::WindowState;

fn test_state() -> WindowState {
    let setup = DeskSetup {
        preferences: Preferences::default(),
        slot: Slot::new(None, 0),
        notifier: Arc::new(|_| {}),
        appearance: Appearance::Dark,
        icon_dir: None,
    };
    WindowState::new(setup, (1180.0, 740.0), 1.0).unwrap()
}

#[test]
fn working_mark_stops_on_ai_idle_and_offline_marks_pane() {
    let mut state = test_state();
    let id = SessionId(1);
    state.desk.workspace.open_instance_with(vec![id], None);
    state.relayout();
    let mut screen = Screen::new(24, 80, 100);
    screen.osc(&[b"133", b"C"]);
    assert!(screen.program_working());
    let working_now = screen.program_working();
    state
        .frame
        .mark_working(&state.desk.workspace, |_| working_now);
    assert!(state.frame.shows_work());
    screen.osc(&[b"0", "\u{2733} Claude Code".as_bytes()]);
    assert!(!screen.program_working());
    let idle_now = screen.program_working();
    state
        .frame
        .mark_working(&state.desk.workspace, |_| idle_now);
    assert!(!state.frame.shows_work());
    state
        .frame
        .mark_offline(&mut state.text, state.desk.strings, |tab| tab == id);
    let pane: &PaneFrame = &state.frame.panes[0];
    assert!(pane.offline.is_some());
    let btn = pane.offline.as_ref().unwrap().button.center();
    assert_eq!(state.frame.hit(btn, false), Some(Hit::Disconnect(id)));
}

#[test]
fn settings_toggle_keep_remote_tasks_and_step_font() {
    let mut state = test_state();
    let host = SshHost {
        target: "me@box".into(),
        port: Some(2222),
    };
    assert!(!state.desk.preferences.keep_remote_tasks);
    assert_eq!(
        state.desk.dial_command(SessionId(1), &host),
        "ssh -p 2222 me@box\r"
    );
    state.apply_setting(SettingControl::KeepRemoteTasks);
    assert!(state.desk.preferences.keep_remote_tasks);
    let kept = state.desk.dial_command(SessionId(1), &host);
    assert!(kept.starts_with("ssh -p 2222 me@box -t 'tmux new-session -A -s trame-"));
    let before = state.desk.preferences.font_size;
    state.apply_setting(SettingControl::TextBigger);
    assert_eq!(state.desk.preferences.font_size, before + 1.0);
}
