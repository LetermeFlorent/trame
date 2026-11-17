use crate::session::SessionId;

use super::{Desk, Effect, Effects};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Seen {
    pub focused: bool,
    pub showing: bool,
}

impl Desk {
    pub fn output(&mut self, id: SessionId, seen: Seen) -> Effects {
        self.power.output(id, self.sessions.resting(id));
        self.sessions.output(id);
        let mut effects = Effects::default();
        if self.sessions.notice_exit(id) {
            effects.push(Effect::Relayout);
        }
        effects.extend(self.clipboard(id, seen));
        effects.extend(self.attention(id, seen));
        effects
    }

    fn clipboard(&self, id: SessionId, seen: Seen) -> Option<Effect> {
        let session = self.sessions.get(id)?;
        let text = session.terminal().screen_mut().take_clipboard()?;
        let focused = seen.focused && self.workspace.active_tab() == Some(id);
        focused.then_some(Effect::Clipboard(text))
    }

    fn attention(&self, id: SessionId, seen: Seen) -> Option<Effect> {
        let session = self.sessions.get(id)?;
        let message = session.terminal().screen_mut().take_attention()?;
        if (seen.focused && seen.showing) || !self.preferences.notify_waiting {
            return None;
        }
        let body = match message.is_empty() {
            true => self.strings.waiting_notice.to_owned(),
            false => message,
        };
        Some(Effect::Notify {
            title: session.label(),
            body,
        })
    }
}
