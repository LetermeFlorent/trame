use winit::keyboard::ModifiersState;

use trame::input::Command;
use trame::term::Terminal;

pub(super) fn takes_key(command: Command, modifiers: ModifiersState, terminal: &Terminal) -> bool {
    command != Command::Copy || !interrupts(modifiers) || terminal.screen().has_selected_text()
}

pub(super) fn after_copy(terminal: &mut Terminal, modifiers: ModifiersState) {
    if interrupts(modifiers) {
        terminal.screen_mut().clear_selection();
    }
}

fn interrupts(modifiers: ModifiersState) -> bool {
    modifiers == ModifiersState::CONTROL
}
