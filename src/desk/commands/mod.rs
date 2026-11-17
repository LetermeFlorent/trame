mod cycle;

use crate::display::TERMINAL_FONT_SIZE;
use crate::input::Command;
use crate::workspace::{Edge, PaneLayout};

use super::{Desk, Effect, Effects, Placement};

impl Desk {
    pub fn command(
        &mut self,
        exiting: bool,
        command: Command,
        panes: &PaneLayout,
    ) -> Option<Effects> {
        let mut effects = Effects::from(Effect::Animate);
        effects.append(match command {
            Command::Copy
            | Command::Paste
            | Command::ScrollPageUp
            | Command::ScrollPageDown
            | Command::ScrollTop
            | Command::ScrollBottom => return None,
            Command::NewTab => self.open(Placement::Tab),
            Command::NewInstance => self.open(Placement::Instance),
            Command::SplitRight => self.open(Placement::Split(Edge::Right)),
            Command::SplitDown => self.open(Placement::Split(Edge::Bottom)),
            Command::CloseTab => self.close_active(exiting),
            Command::ToggleSidebar => {
                self.workspace.sidebar = self.workspace.sidebar.next();
                Effects::default()
            }
            Command::NextTab => self.cycle_tab(1),
            Command::PreviousTab => self.cycle_tab(-1),
            Command::NextInstance => self.cycle_instance(1),
            Command::PreviousInstance => self.cycle_instance(-1),
            Command::FocusPane(edge) => self.focus_neighbor(edge, panes),
            Command::FontBigger => self.store_preferences(self.preferences.step_font_size(1)),
            Command::FontSmaller => self.store_preferences(self.preferences.step_font_size(-1)),
            Command::FontReset => {
                self.store_preferences(self.preferences.with_font_size(TERMINAL_FONT_SIZE))
            }
        });
        effects.push(Effect::Changed);
        Some(effects)
    }

    fn close_active(&mut self, exiting: bool) -> Effects {
        match self.workspace.active_tab() {
            Some(id) => self.close_tab(exiting, id),
            None => Effects::default(),
        }
    }
}
