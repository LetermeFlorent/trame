use std::fmt;

use super::{HostEntry, SshHost};
use crate::input::TextField;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormField {
    Address,
    User,
    Port,
    Secret,
}

impl FormField {
    pub const ALL: [Self; 4] = [Self::Address, Self::User, Self::Port, Self::Secret];

    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Default)]
pub struct ConnectForm {
    fields: [TextField; FormField::ALL.len()],
    focus: usize,
}

impl fmt::Debug for ConnectForm {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConnectForm")
            .field("focus", &self.focus())
            .finish_non_exhaustive()
    }
}

impl ConnectForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filled(entry: &HostEntry, secret: Option<&str>) -> Self {
        let port = entry.port.map(|port| port.to_string()).unwrap_or_default();
        let texts = [
            entry.address.as_str(),
            entry.user.as_deref().unwrap_or_default(),
            port.as_str(),
            secret.unwrap_or_default(),
        ];
        Self {
            fields: texts.map(TextField::new),
            focus: 0,
        }
    }

    pub fn field(&self, field: FormField) -> &TextField {
        &self.fields[field.index()]
    }

    pub fn focus(&self) -> FormField {
        FormField::ALL[self.focus]
    }

    pub fn focus_on(&mut self, field: FormField) {
        self.focus = field.index();
    }

    pub fn next(&mut self) {
        self.focus = (self.focus + 1) % self.fields.len();
    }

    pub fn previous(&mut self) {
        self.focus = (self.focus + self.fields.len() - 1) % self.fields.len();
    }

    pub fn focused_mut(&mut self) -> &mut TextField {
        &mut self.fields[self.focus]
    }

    pub fn host(&self) -> Option<SshHost> {
        SshHost::from_parts(
            self.text(FormField::Address),
            self.text(FormField::User),
            self.text(FormField::Port),
        )
    }

    pub fn secret(&self) -> Option<&str> {
        Some(self.text(FormField::Secret)).filter(|secret| !secret.is_empty())
    }

    pub fn first_invalid(&self) -> Option<FormField> {
        let address = self.text(FormField::Address);
        match () {
            _ if self.host().is_some() => None,
            _ if SshHost::from_parts(address, "", "").is_none() => Some(FormField::Address),
            _ if SshHost::from_parts(address, self.text(FormField::User), "").is_none() => {
                Some(FormField::User)
            }
            _ => Some(FormField::Port),
        }
    }

    fn text(&self, field: FormField) -> &str {
        self.field(field).text()
    }
}
