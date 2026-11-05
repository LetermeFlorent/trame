use std::path::PathBuf;

use trame::locale::LanguageChoice;
use trame::preferences::{Preferences, SshHost};
use trame::theme::Swatch;
use trame::ui::{Density, PanelSide};
use trame::workspace::{NodeSnapshot, Numbering, PaneSnapshot, TabSnapshot};

use crate::snapshot::{restore, single, through_json};

fn every_preference_changed() -> Preferences {
    Preferences {
        font_size: 18.0,
        restore_text: false,
        restore_instances: false,
        restore_tabs: false,
        instance_tabs: 3,
        history_lines: 5_000,
        startup_command: Some("cd work".to_owned()),
        interface_size: 120,
        language: LanguageChoice::French,
        default_folder: Some(PathBuf::from("projects")),
        ssh_hosts: SshHost::parse("me@box:2222").into_iter().collect(),
        confirm_close: false,
        confirm_paste: false,
        detect_links: false,
        custom_colors: vec![Swatch::custom(0x12ab34)],
        panel_side: PanelSide::Right,
        density: Density::Spacious,
        efficiency: false,
        rest_hidden_tabs: false,
        notify_waiting: false,
        program_clipboard: false,
        update_on_start: true,
        explorer_menu: false,
        numbering: Numbering {
            shown: false,
            on_close: false,
            on_move: true,
        },
        shortcuts: serde_json::from_value(serde_json::json!({ "copy": ["Ctrl+Insert"] }))
            .expect("reads the keys"),
    }
}

#[test]
fn preferences_and_text_keys_survive_json() {
    let tab = TabSnapshot {
        transcript: Some("abc-1".to_owned()),
        ..TabSnapshot::default()
    };
    let mut snapshot = single(NodeSnapshot::Pane(PaneSnapshot {
        tabs: vec![tab],
        active: 0,
    }));
    snapshot.preferences = every_preference_changed();
    let parsed = through_json(&snapshot);
    assert_eq!(parsed, snapshot);
    let (_, seen) = restore(&parsed);
    assert_eq!(seen[0].transcript.as_deref(), Some("abc-1"));
}
