mod launch;
mod ssh;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::locale::LanguageChoice;
use crate::ui::metrics::{MAX_TERMINAL_FONT_SIZE, MIN_TERMINAL_FONT_SIZE, TERMINAL_FONT_SIZE};
pub use launch::{MAX_INSTANCE_TABS, MIN_INSTANCE_TABS, Recall};
use launch::{clamp_instance_tabs, sanitized_command};
use ssh::sanitized_hosts;
pub use ssh::{
    ANSWER_WINDOW, ConnectForm, FormField, MAX_SSH_HOSTS, Prompt, SecretAnswer, SshConfig, SshHost,
    aliases_in, appended, menu_hosts, prompt_in, vault,
};

pub const FONT_SIZE_STEP: f32 = 1.0;
pub const INTERFACE_SIZE: u16 = 100;
pub const MIN_INTERFACE_SIZE: u16 = 80;
pub const MAX_INTERFACE_SIZE: u16 = 150;
pub const INTERFACE_SIZE_STEP: u16 = 10;

const PERCENT: f32 = 100.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    pub font_size: f32,
    #[serde(default = "enabled")]
    pub restore_text: bool,
    #[serde(default = "enabled")]
    pub restore_instances: bool,
    #[serde(default = "enabled")]
    pub restore_tabs: bool,
    #[serde(default = "instance_tabs")]
    pub instance_tabs: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_command: Option<String>,
    #[serde(default = "interface_size")]
    pub interface_size: u16,
    #[serde(default)]
    pub language: LanguageChoice,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_folder: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ssh_hosts: Vec<SshHost>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            font_size: TERMINAL_FONT_SIZE,
            restore_text: true,
            restore_instances: true,
            restore_tabs: true,
            instance_tabs: MIN_INSTANCE_TABS,
            startup_command: None,
            interface_size: INTERFACE_SIZE,
            language: LanguageChoice::Auto,
            default_folder: None,
            ssh_hosts: Vec::new(),
        }
    }
}

impl Preferences {
    pub fn sanitized(self) -> Self {
        Self {
            font_size: clamp_font_size(self.font_size),
            interface_size: clamp_interface_size(self.interface_size),
            instance_tabs: clamp_instance_tabs(self.instance_tabs),
            startup_command: sanitized_command(self.startup_command),
            ssh_hosts: sanitized_hosts(self.ssh_hosts),
            ..self
        }
    }

    pub fn with(&self, change: impl FnOnce(&mut Self)) -> Self {
        let mut next = self.clone();
        change(&mut next);
        next
    }

    pub fn with_font_size(&self, font_size: f32) -> Self {
        self.with(|next| next.font_size = clamp_font_size(font_size))
    }

    pub fn step_font_size(&self, steps: i32) -> Self {
        self.with_font_size(self.font_size + steps as f32 * FONT_SIZE_STEP)
    }

    pub fn step_interface_size(&self, steps: i32) -> Self {
        let wanted = i32::from(self.interface_size) + steps * i32::from(INTERFACE_SIZE_STEP);
        let size = wanted.clamp(0, i32::from(u16::MAX)) as u16;
        self.with(|next| next.interface_size = clamp_interface_size(size))
    }

    pub fn ui_scale(&self, window_scale: f32) -> f32 {
        window_scale * f32::from(self.interface_size) / PERCENT
    }

    pub fn font_pixels(&self, window_scale: f32) -> f32 {
        self.font_size * window_scale
    }
}

fn enabled() -> bool {
    true
}

fn instance_tabs() -> u8 {
    MIN_INSTANCE_TABS
}

fn interface_size() -> u16 {
    INTERFACE_SIZE
}

fn clamp_font_size(size: f32) -> f32 {
    match size.is_finite() {
        true => size.clamp(MIN_TERMINAL_FONT_SIZE, MAX_TERMINAL_FONT_SIZE),
        false => TERMINAL_FONT_SIZE,
    }
}

fn clamp_interface_size(size: u16) -> u16 {
    let step = INTERFACE_SIZE_STEP;
    let rounded = size.saturating_add(step / 2) / step * step;
    rounded.clamp(MIN_INTERFACE_SIZE, MAX_INTERFACE_SIZE)
}
