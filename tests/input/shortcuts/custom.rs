use trame::input::{
    ACTIONS, Combo, Command, Keymap, Overrides, ShortcutInput, action_of, default_keys,
};
use trame::ui::Platform;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

use super::{CONTROL, SHIFT};
use crate::character;

const WINDOWS: Platform = Platform::Windows;

fn action(command: Command) -> usize {
    ACTIONS.iter().position(|known| *known == command).unwrap()
}

fn keys(texts: &[&str]) -> Vec<Combo> {
    texts
        .iter()
        .map(|text| Combo::parse(text).unwrap())
        .collect()
}

fn letter(
    keymap: &Keymap,
    modifiers: ModifiersState,
    text: &str,
    code: KeyCode,
) -> Option<Command> {
    let key = character(text);
    let input = ShortcutInput {
        key: &key,
        base: &key,
        physical: PhysicalKey::Code(code),
        modifiers,
    };
    keymap.command_for(input)
}

#[test]
fn a_chosen_key_replaces_the_original_one() {
    let overrides =
        Overrides::default().with_keys(action(Command::NewTab), keys(&["Ctrl+T"]), WINDOWS);
    let keymap = Keymap::new(WINDOWS, &overrides);
    assert_eq!(
        letter(&keymap, CONTROL, "t", KeyCode::KeyT),
        Some(Command::NewTab)
    );
    assert_eq!(letter(&keymap, CONTROL | SHIFT, "t", KeyCode::KeyT), None);
    assert_eq!(keymap.keys(action(Command::NewTab)), keys(&["Ctrl+T"]));
}

#[test]
fn an_action_can_lose_all_its_keys() {
    let new_tab = action(Command::NewTab);
    let overrides = Overrides::default().with_keys(new_tab, Vec::new(), WINDOWS);
    let keymap = Keymap::new(WINDOWS, &overrides);
    assert_eq!(letter(&keymap, CONTROL | SHIFT, "t", KeyCode::KeyT), None);
    assert!(keymap.keys(new_tab).is_empty());
}

#[test]
fn going_back_to_the_original_keys_forgets_the_change() {
    let close = action(Command::CloseTab);
    let changed = Overrides::default().with_keys(close, keys(&["Ctrl+Q"]), WINDOWS);
    assert!(!changed.is_empty());
    let original = default_keys(WINDOWS)[close].clone();
    assert!(changed.with_keys(close, original, WINDOWS).is_empty());
    assert!(changed.reset(close).is_empty());
}

#[test]
fn keeps_at_most_four_different_keys() {
    let texts = ["F1", "F2", "F1", "F3", "F4", "F5"];
    let overrides = Overrides::default().with_keys(0, keys(&texts), WINDOWS);
    assert_eq!(
        overrides.keys(0),
        Some(keys(&["F1", "F2", "F3", "F4"]).as_slice())
    );
}

#[test]
fn names_the_action_that_holds_a_key() {
    let keymap = Keymap::new(WINDOWS, &Overrides::default());
    let close = Combo::parse("Ctrl+Shift+W").unwrap();
    assert_eq!(keymap.owner(&close), Some(action(Command::CloseTab)));
    assert_eq!(keymap.owner(&Combo::parse("F9").unwrap()), None);
}

#[test]
fn a_key_moved_by_hand_leaves_its_old_action() {
    let saved = serde_json::json!({ "close_tab": ["Ctrl+Shift+T"] });
    let overrides: Overrides = serde_json::from_value(saved).unwrap();
    let keymap = Keymap::new(WINDOWS, &overrides);
    let pressed = letter(&keymap, CONTROL | SHIFT, "t", KeyCode::KeyT);
    assert_eq!(pressed, Some(Command::CloseTab));
    assert!(keymap.keys(action(Command::NewTab)).is_empty());
}

#[test]
fn saved_keys_come_back_the_same() {
    let overrides = Overrides::default()
        .with_keys(
            action(Command::Copy),
            keys(&["Ctrl+Insert", "Ctrl+Shift+C"]),
            WINDOWS,
        )
        .with_keys(action(Command::FontBigger), keys(&["Ctrl++"]), WINDOWS);
    let saved = serde_json::to_value(&overrides).unwrap();
    assert_eq!(
        saved["copy"],
        serde_json::json!(["Ctrl+Insert", "Ctrl+Shift+C"])
    );
    let loaded: Overrides = serde_json::from_value(saved).unwrap();
    assert_eq!(loaded, overrides);
}

#[test]
fn skips_saved_keys_it_cannot_read() {
    let saved = serde_json::json!({
        "new_tab": ["Ctrl+Foo", "Ctrl+T", 5],
        "unknown_action": ["F1"],
        "copy": "Ctrl+C",
        "paste": ["Ctrl+Bar"],
        "close_tab": [],
    });
    let overrides: Overrides = serde_json::from_value(saved).unwrap();
    assert_eq!(
        overrides.keys(action(Command::NewTab)),
        Some(keys(&["Ctrl+T"]).as_slice())
    );
    assert_eq!(overrides.keys(action(Command::Copy)), None);
    assert_eq!(overrides.keys(action(Command::Paste)), None);
    assert_eq!(
        overrides.keys(action(Command::CloseTab)),
        Some([].as_slice())
    );
    assert_eq!(action_of("unknown_action"), None);
    let broken: Overrides = serde_json::from_value(serde_json::json!("oops")).unwrap();
    assert!(broken.is_empty());
}
