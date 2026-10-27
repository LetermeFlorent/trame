use crate::input::TextField;

use super::Preferences;

pub const MIN_INSTANCE_TABS: u8 = 1;
pub const MAX_INSTANCE_TABS: u8 = 8;

const ENTER: char = '\r';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recall {
    Instances,
    Tabs,
    Text,
}

impl Recall {
    pub const ALL: [Self; 3] = [Self::Instances, Self::Tabs, Self::Text];
}

impl Preferences {
    pub fn recalls(&self, recall: Recall) -> bool {
        match recall {
            Recall::Instances => self.restore_instances,
            Recall::Tabs => self.restore_instances && self.restore_tabs,
            Recall::Text => self.restore_instances && self.restore_tabs && self.restore_text,
        }
    }

    pub fn can_recall(&self, recall: Recall) -> bool {
        match recall {
            Recall::Instances => true,
            Recall::Tabs => self.restore_instances,
            Recall::Text => self.restore_instances && self.restore_tabs,
        }
    }

    pub fn toggle(&self, recall: Recall) -> Self {
        self.with(|next| {
            let flag = match recall {
                Recall::Instances => &mut next.restore_instances,
                Recall::Tabs => &mut next.restore_tabs,
                Recall::Text => &mut next.restore_text,
            };
            *flag = !*flag;
        })
    }

    pub fn step_instance_tabs(&self, steps: i32) -> Self {
        let wanted = i32::from(self.instance_tabs) + steps;
        let count = wanted.clamp(0, i32::from(u8::MAX)) as u8;
        self.with(|next| next.instance_tabs = clamp_instance_tabs(count))
    }

    pub fn with_startup_command(&self, text: &str) -> Self {
        self.with(|next| next.startup_command = sanitized_command(Some(text.to_owned())))
    }

    pub fn startup_line(&self) -> Option<String> {
        self.startup_command
            .as_ref()
            .map(|command| format!("{command}{ENTER}"))
    }
}

pub(super) fn instance_tabs() -> u8 {
    MIN_INSTANCE_TABS
}

pub(super) fn clamp_instance_tabs(count: u8) -> u8 {
    count.clamp(MIN_INSTANCE_TABS, MAX_INSTANCE_TABS)
}

pub(super) fn sanitized_command(command: Option<String>) -> Option<String> {
    let field = TextField::new(command?.trim());
    let text = field.text().trim();
    (!text.is_empty()).then(|| text.to_owned())
}
