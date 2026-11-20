use std::time::{Duration, Instant};

use gtk::prelude::*;
use gtk::{gdk, gio};
use trame::session::SessionId;

use super::input::clipboard::write_text;
use super::state::WindowState;

const QUIET_FOR: Duration = Duration::from_secs(5);
const NOTICE_ID: &str = "trame-attention";

#[derive(Default)]
pub struct GnomeNotices {
    last: Option<Instant>,
    shown: bool,
}

impl GnomeNotices {
    pub fn forward(
        &mut self,
        state: &WindowState,
        id: SessionId,
        app: Option<&adw::Application>,
        display: Option<&gdk::Display>,
    ) {
        let Some((label, clip, attention)) = take_outputs(state, id) else {
            return;
        };
        let active_focused =
            state.view.window_focused && state.desk.workspace.active_tab() == Some(id);
        if let Some(text) =
            clip.filter(|_| active_focused && state.desk.preferences.program_clipboard)
        {
            write_text(&text, display);
        }
        if let Some(msg) = attention {
            self.send_attention(state, id, (&label, &msg), app);
        }
    }

    fn send_attention(
        &mut self,
        state: &WindowState,
        id: SessionId,
        info: (&str, &str),
        app: Option<&adw::Application>,
    ) {
        let (label, message) = info;
        let visible =
            state.view.window_focused && state.frame.panes.iter().any(|p| p.active == Some(id));
        if visible || !state.desk.preferences.notify_waiting {
            return;
        }
        let now = Instant::now();
        if self.last.is_some_and(|last| now < last + QUIET_FOR) {
            return;
        }
        self.last = Some(now);
        let body = if message.is_empty() {
            state.desk.strings.waiting_notice
        } else {
            message
        };
        if let Some(app) = app {
            let note = gio::Notification::new(label);
            note.set_body(Some(body));
            app.send_notification(Some(NOTICE_ID), &note);
            self.shown = true;
        }
    }

    pub fn clear(&mut self, app: Option<&adw::Application>) {
        if self.shown
            && let Some(app) = app
        {
            app.withdraw_notification(NOTICE_ID);
            self.shown = false;
        }
    }
}

fn take_outputs(
    state: &WindowState,
    id: SessionId,
) -> Option<(String, Option<String>, Option<String>)> {
    let session = state.desk.sessions.get(id)?;
    let label = session.label();
    let mut terminal = session.terminal();
    let clip = terminal.screen_mut().take_clipboard();
    let attention = terminal.screen_mut().take_attention();
    Some((label, clip, attention))
}
