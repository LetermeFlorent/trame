use trame::locale::Language;
use trame::preferences::{ConnectForm, Preferences, SshHost};
use trame::ui::Hit;
use trame::ui::frame::{HostsView, SettingControl, SettingsRequest, SettingsTab};

use super::{hosts, opened, page};
use crate::sidebar::settings::{placed, request_on};
use crate::text_system;

fn opened_with(preferences: &Preferences, saved: &[SshHost]) -> trame::ui::FrameLayout {
    let view = HostsView {
        hosts: saved,
        first: 0,
        form: None,
        changing: false,
    };
    let request = SettingsRequest {
        hosts: Some(view),
        ..request_on(
            preferences,
            Language::English.strings(),
            SettingsTab::Connections,
        )
    };
    placed(&mut text_system(), &request)
}

#[test]
fn the_keep_remote_tasks_switch_is_off_by_default_and_follows_its_preference() {
    let off = opened(&hosts(2), 0, None);
    let view = page(&off);
    let row = view.toggle.as_ref().expect("the keep remote tasks switch");
    assert!(!row.on && row.enabled);
    assert_eq!(row.control, SettingControl::KeepRemoteTasks);
    assert!(view.header.line.rect.bottom() <= row.line.rect.y);
    let list = view.list.as_ref().expect("a host list");
    assert!(row.line.rect.bottom() <= list.rect.y);
    assert_eq!(
        off.hit(row.line.rect.center(), false),
        Some(Hit::Setting(SettingControl::KeepRemoteTasks))
    );
    let on = Preferences {
        keep_remote_tasks: true,
        ..Preferences::default()
    };
    let layout = opened_with(&on, &[]);
    let toggle = page(&layout).toggle.as_ref().expect("the switch");
    assert!(toggle.on);
    let form = ConnectForm::new();
    let editing = opened(&hosts(2), 0, Some(&form));
    assert!(page(&editing).toggle.is_none());
}
