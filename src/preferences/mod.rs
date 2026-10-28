mod closing;
mod colors;
mod history;
mod launch;
mod sizes;
mod ssh;
mod updates;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::input::Overrides;
use crate::locale::LanguageChoice;
use crate::theme::Swatch;
use crate::ui::display::{Density, PanelSide};
use crate::ui::metrics::TERMINAL_FONT_SIZE;
use crate::workspace::Numbering;
pub use closing::CONFIRM_FROM_TABS;
pub use colors::MAX_CUSTOM_COLORS;
use colors::sanitized_colors;
pub use history::{DEFAULT_HISTORY_LINES, MAX_HISTORY_LINES, MIN_HISTORY_LINES};
use history::{history_lines, sanitized_history};
pub use launch::{MAX_INSTANCE_TABS, MIN_INSTANCE_TABS, Recall};
use launch::{clamp_instance_tabs, instance_tabs, sanitized_command};
pub use sizes::{MAX_INTERFACE_SIZE, MIN_INTERFACE_SIZE};
use sizes::{clamp_font_size, clamp_interface_size, interface_size};
use ssh::sanitized_hosts;
pub use ssh::{
    ANSWER_WINDOW, ConnectForm, FormField, HostEntry, Prompt, SecretAnswer, SshConfig, SshHost,
    SshLogin, aliases_in, appended, entry_in, menu_hosts, prompt_in, vault, with_host,
    without_host, written_by_hand,
};
pub use updates::Updates;

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
    #[serde(default = "history_lines")]
    pub history_lines: usize,
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
    #[serde(default = "enabled")]
    pub confirm_close: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_colors: Vec<Swatch>,
    #[serde(default)]
    pub panel_side: PanelSide,
    #[serde(default)]
    pub density: Density,
    #[serde(default = "enabled")]
    pub efficiency: bool,
    #[serde(default = "enabled")]
    pub rest_hidden_tabs: bool,
    #[serde(default = "enabled")]
    pub notify_waiting: bool,
    #[serde(default = "enabled")]
    pub program_clipboard: bool,
    #[serde(default, skip_serializing_if = "Overrides::is_empty")]
    pub shortcuts: Overrides,
    #[serde(flatten)]
    pub numbering: Numbering,
    #[serde(flatten)]
    pub updates: Updates,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            font_size: TERMINAL_FONT_SIZE,
            restore_text: true,
            restore_instances: true,
            restore_tabs: true,
            instance_tabs: MIN_INSTANCE_TABS,
            history_lines: DEFAULT_HISTORY_LINES,
            startup_command: None,
            interface_size: interface_size(),
            language: LanguageChoice::Auto,
            default_folder: None,
            ssh_hosts: Vec::new(),
            confirm_close: true,
            custom_colors: Vec::new(),
            panel_side: PanelSide::Left,
            density: Density::Compact,
            efficiency: true,
            rest_hidden_tabs: true,
            notify_waiting: true,
            program_clipboard: true,
            shortcuts: Overrides::default(),
            numbering: Numbering::default(),
            updates: Updates::default(),
        }
    }
}

impl Preferences {
    pub fn sanitized(self) -> Self {
        Self {
            font_size: clamp_font_size(self.font_size),
            interface_size: clamp_interface_size(self.interface_size),
            instance_tabs: clamp_instance_tabs(self.instance_tabs),
            history_lines: sanitized_history(self.history_lines),
            startup_command: sanitized_command(self.startup_command),
            ssh_hosts: sanitized_hosts(self.ssh_hosts),
            custom_colors: sanitized_colors(self.custom_colors),
            ..self
        }
    }

    pub fn with(&self, change: impl FnOnce(&mut Self)) -> Self {
        let mut next = self.clone();
        change(&mut next);
        next
    }

    pub fn power_switches(&self) -> [bool; 2] {
        [self.efficiency, self.rest_hidden_tabs]
    }
}

fn enabled() -> bool {
    true
}
