use crate::locale::{ALIAS_SLOT, COUNT_SLOT};

use super::Closing;
use crate::desk::Desk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confirm {
    Close,
    Restart,
    Remove,
    Paste,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    pub title: String,
    pub body: String,
    pub confirm: Confirm,
}

impl Desk {
    pub fn question(&self) -> Option<Question> {
        let strings = self.strings;
        let closing = self.closing.as_ref()?;
        if let Closing::Forget(host) = closing {
            return Some(Question {
                title: strings.remove_question.replace(ALIAS_SLOT, host.alias()?),
                body: strings.hand_written.to_owned(),
                confirm: Confirm::Remove,
            });
        }
        if let Closing::Paste(pending) = closing {
            return Some(Question {
                title: strings.paste_question.to_owned(),
                body: pending.body.clone(),
                confirm: Confirm::Paste,
            });
        }
        if let Closing::Update = closing {
            let offer = self.update.offer.as_ref()?;
            return Some(Question {
                title: offer.question(strings),
                body: strings.update_restarts.to_owned(),
                confirm: Confirm::Restart,
            });
        }
        let count = self.tabs_closing(closing);
        let title = match closing {
            Closing::Instance(_) => strings.close_instance,
            _ => strings.close_window,
        };
        (count > 0).then(|| Question {
            title: title.to_owned(),
            body: strings
                .tabs_will_close
                .replace(COUNT_SLOT, &count.to_string()),
            confirm: Confirm::Close,
        })
    }
}
