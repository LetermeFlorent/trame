use trame::input::Command;
use trame::workspace::PaneLayout;

use crate::app::running::Running;
use crate::app::running::pointer::Scroll;

impl Running {
    pub(in crate::app::running) fn command(&mut self, exiting: bool, command: Command) {
        let panes = self.pane_layout();
        match self.desk.command(exiting, command, &panes) {
            Some(effects) => self.apply(effects),
            None => {
                self.motion.animate_next();
                self.view_command(command);
                self.changed();
            }
        }
    }

    fn view_command(&mut self, command: Command) {
        match command {
            Command::Copy => self.copy_shortcut(),
            Command::Paste => self.paste(),
            Command::ScrollPageUp => self.scroll_active(Scroll::Pages(1)),
            Command::ScrollPageDown => self.scroll_active(Scroll::Pages(-1)),
            Command::ScrollTop => self.scroll_active(Scroll::Top),
            Command::ScrollBottom => self.scroll_active(Scroll::Bottom),
            _ => {}
        }
    }

    fn pane_layout(&self) -> PaneLayout {
        PaneLayout {
            panes: self
                .frame
                .panes
                .iter()
                .map(|pane| (pane.id, pane.rect))
                .collect(),
            dividers: Vec::new(),
        }
    }
}
