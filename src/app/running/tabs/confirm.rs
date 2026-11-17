use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};

use trame::desk::{Closing, Confirm};
use trame::ui::Hit;
use trame::ui::frame::{FormAction, MessageRequest};

use crate::app::running::Running;
use crate::app::running::keys::is_escape;

const CLOSE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Close];
const UPDATE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Restart];
const REMOVE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Remove];
const PASTE_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Paste];

impl Running {
    pub(in crate::app::running) fn ask_close(&mut self, closing: Closing) {
        let effects = self.desk.ask_close(closing, self.loader.is_some());
        self.apply(effects);
    }

    pub(in crate::app::running) fn place_message(&mut self) {
        let Some(question) = self.desk.question() else {
            self.desk.closing = None;
            return;
        };
        let lines: Vec<&str> = question.body.lines().collect();
        let request = MessageRequest {
            title: &question.title,
            lines: &lines,
            actions: actions_for(question.confirm),
            strings: self.desk.strings,
        };
        self.frame.place_message(&mut self.text, &request);
    }

    pub(in crate::app::running) fn closing_button(&mut self, state: ElementState) -> bool {
        if self.desk.closing.is_none() {
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
            self.settle_closing(confirmed);
        }
        true
    }

    pub(in crate::app::running) fn closing_key(&mut self, event: &KeyEvent) -> bool {
        if self.desk.closing.is_none() {
            return false;
        }
        let confirmed = event.logical_key == Key::Named(NamedKey::Enter);
        if confirmed || is_escape(event) {
            self.settle_closing(confirmed);
        }
        true
    }

    fn settle_closing(&mut self, confirmed: bool) {
        let effects = self.desk.settle_closing(confirmed);
        self.apply(effects);
    }
}

fn actions_for(confirm: Confirm) -> &'static [FormAction] {
    match confirm {
        Confirm::Close => &CLOSE_ACTIONS,
        Confirm::Restart => &UPDATE_ACTIONS,
        Confirm::Remove => &REMOVE_ACTIONS,
        Confirm::Paste => &PASTE_ACTIONS,
    }
}
