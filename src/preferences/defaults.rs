use crate::display::Density;
use crate::display::PanelSide;
use crate::input::Overrides;
use crate::locale::LanguageChoice;
use crate::workspace::Numbering;

use super::DEFAULT_HISTORY_LINES;
use super::MIN_INSTANCE_TABS;
use super::Preferences;
use super::TERMINAL_FONT_SIZE;
use super::interface_size;

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
            confirm_paste: true,
            detect_links: true,
            smooth_scroll: true,
            custom_colors: Vec::new(),
            panel_side: PanelSide::Left,
            density: Density::Compact,
            efficiency: true,
            rest_hidden_tabs: true,
            notify_waiting: true,
            program_clipboard: true,
            update_on_start: true,
            explorer_menu: false,
            keep_remote_tasks: false,
            shortcuts: Overrides::default(),
            numbering: Numbering::default(),
        }
    }
}
