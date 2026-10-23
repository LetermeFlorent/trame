use trame::preferences::{Preferences, SshHost};
use trame::ui::Hit;
use trame::ui::frame::SettingControl;

use super::{english, panel};

fn with_hosts(targets: &[&str]) -> Preferences {
    let ssh_hosts = targets
        .iter()
        .map(|target| SshHost::parse(target).expect("a valid connection"))
        .collect();
    Preferences {
        ssh_hosts,
        ..Preferences::default()
    }
}

#[test]
fn without_saved_connections_the_section_explains_how_to_add_one() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    assert!(settings.hosts.is_empty());
    let hint = settings.hosts_hint.as_ref().expect("a hint");
    assert!(settings.popup.rect.contains(hint.rect.center()));
    assert_eq!(
        layout.hit(hint.rect.center(), false),
        Some(Hit::SettingsPanel)
    );
}

#[test]
fn each_saved_connection_has_a_line_and_a_button_to_forget_it() {
    let layout = english(&with_hosts(&["me@box:2222", "server", "root@10.0.0.2"]));
    let settings = panel(&layout);
    assert!(settings.hosts_hint.is_none());
    assert_eq!(settings.hosts.len(), 3);
    for (index, row) in settings.hosts.iter().enumerate() {
        assert!(row.chips.is_empty());
        let (forget, control) = row.clear.expect("a forget button");
        assert_eq!(control, SettingControl::ForgetHost(index));
        assert!(settings.popup.rect.contains(forget.center()));
        assert_eq!(
            layout.hit(forget.center(), false),
            Some(Hit::Setting(control))
        );
    }
    for pair in settings.hosts.windows(2) {
        assert!(pair[0].line.rect.bottom() <= pair[1].line.rect.y);
    }
}

#[test]
fn the_window_grows_with_the_saved_connections_and_stays_centred() {
    let one = english(&with_hosts(&["a"]));
    let empty = english(&Preferences::default());
    assert_eq!(
        panel(&one).popup.rect.height,
        panel(&empty).popup.rect.height
    );
    let three = english(&with_hosts(&["a", "b", "c"]));
    let (small, tall) = (panel(&one), panel(&three));
    let line = small.hosts[0].line.rect.height;
    assert!((tall.popup.rect.height - small.popup.rect.height - 2.0 * line).abs() <= 1.0);
    let last = tall.hosts[2].line.rect;
    assert!(last.bottom() <= tall.popup.rect.bottom());
    let (panel_mid, backdrop_mid) = (tall.popup.rect.center(), tall.popup.backdrop.center());
    assert!((panel_mid.y - backdrop_mid.y).abs() <= 1.0);
}
