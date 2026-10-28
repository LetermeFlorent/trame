use trame::input::TextField;
use trame::preferences::Preferences;
use trame::ui::frame::SettingControl;

use crate::app::running::{FolderTarget, Running};

impl Running {
    pub(in crate::app::running) fn apply_setting(&mut self, control: SettingControl) {
        let current = &self.preferences;
        let next = match control {
            SettingControl::Close => return self.toggle_settings(),
            SettingControl::Tab(tab) => return self.show_tab(tab),
            SettingControl::AddHost => return self.add_host(),
            SettingControl::Keys(key) => return self.key_setting(key),
            SettingControl::Recall(recall) => current.toggle(recall),
            SettingControl::ConfirmClose => current.toggle_confirm_close(),
            SettingControl::Efficiency
            | SettingControl::RestHiddenTabs
            | SettingControl::NotifyWaiting
            | SettingControl::AutoUpdate => current.with(|next| flip(next, control)),
            SettingControl::Numbers(choice) => {
                current.with(|next| choice.toggle(&mut next.numbering))
            }
            SettingControl::StartupCommand | SettingControl::HistoryLines => {
                return self.edit_setting(control);
            }
            SettingControl::ClearStartupCommand => current.with_startup_command(""),
            SettingControl::InstanceTabsFewer => current.step_instance_tabs(-1),
            SettingControl::InstanceTabsMore => current.step_instance_tabs(1),
            SettingControl::HistoryFewer => current.step_history_lines(-1),
            SettingControl::HistoryMore => current.step_history_lines(1),
            SettingControl::TextSmaller => current.step_font_size(-1),
            SettingControl::TextBigger => current.step_font_size(1),
            SettingControl::InterfaceSmaller => current.step_interface_size(-1),
            SettingControl::InterfaceBigger => current.step_interface_size(1),
            SettingControl::Language(language) => current.with(|next| next.language = language),
            SettingControl::PanelSide(side) => current.with(|next| next.panel_side = side),
            SettingControl::Density(density) => current.with(|next| next.density = density),
            SettingControl::DefaultFolder => {
                let start = self.preferences.default_folder.clone();
                return self.ask_folder(FolderTarget::Default, start);
            }
            SettingControl::ClearFolder => current.with(|next| next.default_folder = None),
        };
        self.set_preferences(next);
    }

    fn edit_setting(&mut self, control: SettingControl) {
        let field = match control {
            SettingControl::HistoryLines => {
                let mut field = TextField::new(&self.preferences.history_lines.to_string());
                field.select_all();
                field
            }
            _ => TextField::new(
                self.preferences
                    .startup_command
                    .as_deref()
                    .unwrap_or_default(),
            ),
        };
        self.setting_field = Some((control, field));
        self.changed();
    }
}

fn flip(next: &mut Preferences, control: SettingControl) {
    let switch = match control {
        SettingControl::Efficiency => &mut next.efficiency,
        SettingControl::RestHiddenTabs => &mut next.rest_hidden_tabs,
        SettingControl::NotifyWaiting => &mut next.notify_waiting,
        SettingControl::AutoUpdate => &mut next.updates.auto,
        _ => return,
    };
    *switch = !*switch;
}
