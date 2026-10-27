use crate::input::{Combo, Trigger};
use crate::locale::KeyStrings;
use crate::ui::Platform;

pub fn key_label(combo: &Combo, strings: &KeyStrings, platform: Platform) -> String {
    let alt = match platform {
        Platform::Mac => "Option",
        _ => "Alt",
    };
    let logo = match platform {
        Platform::Mac => "Cmd",
        Platform::Windows => "Win",
        Platform::Linux => "Super",
    };
    let modifiers = ["Ctrl", strings.shift, alt, logo];
    let mut parts: Vec<String> = modifiers
        .iter()
        .zip(combo.chord.flags())
        .filter(|(_, on)| *on)
        .map(|(name, _)| name.to_string())
        .collect();
    parts.push(trigger_label(combo.trigger, strings));
    parts.join("+")
}

fn trigger_label(trigger: Trigger, strings: &KeyStrings) -> String {
    trigger
        .named_slot()
        .and_then(|slot| strings.named.get(slot))
        .map_or_else(|| trigger.text(), |name| name.to_string())
}
