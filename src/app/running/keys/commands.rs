use winit::event_loop::ActiveEventLoop;

use trame::input::Command;
use trame::ui::metrics::TERMINAL_FONT_SIZE;
use trame::workspace::{Edge, Instance, PaneLayout};

use crate::app::running::Running;
use crate::app::running::pointer::Scroll;
use crate::app::running::tabs::Placement;

impl Running {
    pub(in crate::app::running) fn command(
        &mut self,
        event_loop: &ActiveEventLoop,
        command: Command,
    ) {
        self.motion.animate_next();
        match command {
            Command::NewTab => self.open_or_report(event_loop, Placement::Tab),
            Command::NewInstance => self.open_or_report(event_loop, Placement::Instance),
            Command::SplitRight => self.open_or_report(event_loop, Placement::Split(Edge::Right)),
            Command::SplitDown => self.open_or_report(event_loop, Placement::Split(Edge::Bottom)),
            Command::CloseTab => {
                if let Some(id) = self.workspace.active_tab() {
                    self.close_tab(event_loop, id);
                }
            }
            Command::ToggleSidebar => self.workspace.sidebar_open = !self.workspace.sidebar_open,
            Command::Copy => self.copy_shortcut(),
            Command::Paste => self.paste(),
            Command::NextTab => self.cycle_tab(1),
            Command::PreviousTab => self.cycle_tab(-1),
            Command::SelectTab(index) => self.select_tab(Some(index)),
            Command::LastTab => self.select_tab(None),
            Command::NextInstance => self.cycle_instance(1),
            Command::PreviousInstance => self.cycle_instance(-1),
            Command::FocusPane(edge) => self.focus_neighbor(edge),
            Command::FontBigger => self.set_preferences(self.preferences.step_font_size(1)),
            Command::FontSmaller => self.set_preferences(self.preferences.step_font_size(-1)),
            Command::FontReset => {
                self.set_preferences(self.preferences.with_font_size(TERMINAL_FONT_SIZE))
            }
            Command::ScrollPageUp => self.scroll_active(Scroll::Pages(1)),
            Command::ScrollPageDown => self.scroll_active(Scroll::Pages(-1)),
            Command::ScrollTop => self.scroll_active(Scroll::Top),
            Command::ScrollBottom => self.scroll_active(Scroll::Bottom),
        }
        self.changed();
    }

    fn cycle_tab(&mut self, step: isize) {
        if let Some(pane) = self
            .workspace
            .active_instance_mut()
            .and_then(Instance::active_mut)
        {
            pane.cycle(step);
        }
    }

    fn select_tab(&mut self, index: Option<usize>) {
        let Some(pane) = self
            .workspace
            .active_instance_mut()
            .and_then(Instance::active_mut)
        else {
            return;
        };
        let index = index.unwrap_or(pane.tabs.len().saturating_sub(1));
        if index < pane.tabs.len() {
            pane.active = index;
        }
    }

    fn cycle_instance(&mut self, step: isize) {
        let count = self.workspace.instances.len() as isize;
        if count > 1 {
            let next = (self.workspace.active as isize + step).rem_euclid(count);
            self.workspace.active = next as usize;
        }
    }

    fn focus_neighbor(&mut self, edge: Edge) {
        let layout = PaneLayout {
            panes: self
                .frame
                .panes
                .iter()
                .map(|pane| (pane.id, pane.rect))
                .collect(),
            dividers: Vec::new(),
        };
        let Some(instance) = self.workspace.active_instance_mut() else {
            return;
        };
        if let Some(next) = layout.neighbor(instance.active_pane, edge) {
            instance.focus(next);
        }
    }
}
