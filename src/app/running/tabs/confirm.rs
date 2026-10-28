use winit::event::{ElementState, KeyEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};

use trame::locale::ALIAS_SLOT;
use trame::ui::Hit;
use trame::ui::frame::{FormAction, MessageRequest};

use super::Closing;
use crate::app::running::Running;
use crate::app::running::keys::is_escape;

const CLOSE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Close];
const UPDATE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Restart];
const REMOVE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Remove];
const PASTE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Paste];

struct Question {
    title: String,
    body: String,
    actions: &'static [FormAction],
}

impl Running {
    pub(in crate::app::running) fn place_message(&mut self) {
        let Some(closing) = self.closing.clone() else {
            return;
        };
        let Some(question) = self.question(closing) else {
            self.closing = None;
            return;
        };
        let lines: Vec<&str> = question.body.lines().collect();
        let request = MessageRequest {
            title: &question.title,
            lines: &lines,
            actions: question.actions,
            strings: self.strings,
        };
        self.frame.place_message(&mut self.text, &request);
    }

    fn question(&mut self, closing: Closing) -> Option<Question> {
        let strings = self.strings;
        if let Closing::Forget(index) = closing {
            let host = self.listed_host(index)?;
            return Some(Question {
                title: strings.remove_question.replace(ALIAS_SLOT, host.alias()?),
                body: strings.hand_written.to_owned(),
                actions: &REMOVE_ACTIONS,
            });
        }
        if let Closing::Paste(pending) = &closing {
            return Some(Question {
                title: strings.paste_question.to_owned(),
                body: pending.body.clone(),
                actions: &PASTE_ACTIONS,
            });
        }
        if let Closing::Update = closing {
            let offer = self.update.offer.as_ref()?;
            return Some(Question {
                title: offer.question(strings),
                body: strings.update_restarts.to_owned(),
                actions: &UPDATE_ACTIONS,
            });
        }
        let count = self.tabs_closing(&closing);
        let title = match closing {
            Closing::Instance(_) => strings.close_instance,
            _ => strings.close_window,
        };
        (count > 0).then(|| Question {
            title: title.to_owned(),
            body: format!("{count} {}", strings.tabs_will_close),
            actions: &CLOSE_ACTIONS,
        })
    }

    pub(in crate::app::running) fn closing_button(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
    ) -> bool {
        if self.closing.is_none() {
            return false;
        }
        let hit = self.hit_under_pointer();
        if !matches!(
            hit,
            Some(Hit::FormButton(_) | Hit::EditClose | Hit::Dialog | Hit::Backdrop)
        ) {
            return false;
        }
        if state == ElementState::Pressed && hit != Some(Hit::Dialog) {
            let confirmed = matches!(
                hit,
                Some(Hit::FormButton(
                    FormAction::Close
                        | FormAction::Restart
                        | FormAction::Remove
                        | FormAction::Paste
                ))
            );
            self.settle_closing(event_loop, confirmed);
        }
        true
    }

    pub(in crate::app::running) fn closing_key(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: &KeyEvent,
    ) -> bool {
        if self.closing.is_none() {
            return false;
        }
        let confirmed = event.logical_key == Key::Named(NamedKey::Enter);
        if confirmed || is_escape(event) {
            self.settle_closing(event_loop, confirmed);
        }
        true
    }
}
