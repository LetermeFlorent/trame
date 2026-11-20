use std::sync::Arc;

use gtk::gdk;
use trame::desk::{Closing, Desk, DeskSetup, Grids, Placement};
use trame::display::Platform;
use trame::host::persist::Slot;
use trame::input::{Command, Keymap, Overrides, Pressed, asks_before_pasting, pressed};
use trame::preferences::Preferences;
use trame::term::{Modes, SelectionKind, Terminal};
use trame::theme::Appearance;
use trame::ui::Hit;
use trame::workspace::PaneId;
use winit::keyboard::ModifiersState;

use super::drop_scroll::{DropOutcome, drop_target};
use super::keys::{DecodedKey, takes_key};

#[test]
fn gdk_keys_map_shortcuts_azerty_digits_and_copy_guard() {
    let keymap = Keymap::new(Platform::Linux, &Overrides::default());
    let ctrl_shift = gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::SHIFT_MASK;
    let new_tab = DecodedKey::from_gdk(gdk::Key::T, 28, ctrl_shift).unwrap();
    assert_eq!(
        keymap.command_for(new_tab.shortcut_input()),
        Some(Command::NewTab)
    );
    let azerty_zero =
        DecodedKey::from_gdk(gdk::Key::agrave, 19, gdk::ModifierType::CONTROL_MASK).unwrap();
    assert_eq!(
        keymap.command_for(azerty_zero.shortcut_input()),
        Some(Command::FontReset)
    );
    let shift_only =
        DecodedKey::from_gdk(gdk::Key::Shift_L, 50, gdk::ModifierType::SHIFT_MASK).unwrap();
    assert_eq!(
        pressed(shift_only.shortcut_input(), Platform::Linux),
        Pressed::Modifier
    );
    let mut term = Terminal::new(4, 20, 100);
    term.feed(b"hello world");
    assert!(!takes_key(Command::Copy, ModifiersState::CONTROL, &term));
    term.screen_mut().start_selection(SelectionKind::Line, 0, 0);
    assert!(takes_key(Command::Copy, ModifiersState::CONTROL, &term));
}

#[test]
fn drop_and_paste_honor_modals_and_confirm_paste() {
    let pane = PaneId(1);
    assert_eq!(
        drop_target(Some(Hit::Terminal(pane)), true),
        DropOutcome::Ignored
    );
    assert_eq!(
        drop_target(Some(Hit::Terminal(pane)), false),
        DropOutcome::Pane(pane)
    );
    let mut desk = Desk::new(DeskSetup {
        preferences: Preferences::default(),
        slot: Slot::new(None, 0),
        notifier: Arc::new(|_| {}),
        appearance: Appearance::Dark,
        icon_dir: None,
    });
    assert!(asks_before_pasting("first\nsecond\n", Modes::default()));
    let _ = desk.open_at(Placement::Instance, None);
    desk.start_queued(&Grids::default());
    let _ = desk.paste("first\nsecond\n".into());
    assert!(matches!(desk.closing, Some(Closing::Paste(_))));
}
