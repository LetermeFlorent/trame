use trame::preferences::Preferences;
use trame::ui::frame::SettingControl;

pub(super) fn preference_for(cur: &Preferences, control: SettingControl) -> Preferences {
    match control {
        SettingControl::Recall(r) => cur.toggle(r),
        SettingControl::ConfirmClose => cur.toggle_confirm_close(),
        SettingControl::Numbers(c) => cur.with(|n| c.toggle(&mut n.numbering)),
        SettingControl::ClearStartupCommand => cur.with_startup_command(""),
        SettingControl::InstanceTabsFewer => cur.step_instance_tabs(-1),
        SettingControl::InstanceTabsMore => cur.step_instance_tabs(1),
        SettingControl::HistoryFewer => cur.step_history_lines(-1),
        SettingControl::HistoryMore => cur.step_history_lines(1),
        SettingControl::TextSmaller => cur.step_font_size(-1),
        SettingControl::TextBigger => cur.step_font_size(1),
        SettingControl::InterfaceSmaller => cur.step_interface_size(-1),
        SettingControl::InterfaceBigger => cur.step_interface_size(1),
        SettingControl::Language(l) => cur.with(|n| n.language = l),
        SettingControl::PanelSide(s) => cur.with(|n| n.panel_side = s),
        SettingControl::Density(d) => cur.with(|n| n.density = d),
        SettingControl::ClearFolder => cur.with(|n| n.default_folder = None),
        _ => cur.with(|n| flip_switch(n, control)),
    }
}

fn flip_switch(next: &mut Preferences, control: SettingControl) {
    let flag = match control {
        SettingControl::ConfirmPaste => &mut next.confirm_paste,
        SettingControl::DetectLinks => &mut next.detect_links,
        SettingControl::SmoothScroll => &mut next.smooth_scroll,
        SettingControl::Efficiency => &mut next.efficiency,
        SettingControl::RestHiddenTabs => &mut next.rest_hidden_tabs,
        SettingControl::NotifyWaiting => &mut next.notify_waiting,
        SettingControl::ProgramClipboard => &mut next.program_clipboard,
        SettingControl::UpdateOnStart => &mut next.update_on_start,
        SettingControl::KeepRemoteTasks => &mut next.keep_remote_tasks,
        _ => return,
    };
    *flag = !*flag;
}
