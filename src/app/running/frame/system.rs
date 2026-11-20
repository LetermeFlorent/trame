#[cfg(test)]
#[path = "system_tests.rs"]
mod tests;

use std::collections::HashSet;
use std::time::Instant;

use trame::host::power::Shown;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::theme::Rgba;

use crate::app::running::Running;
use crate::app::running::pointer::selection::PictureView;

impl Running {
    pub(in crate::app::running) fn sync_fresh(
        fresh: bool,
        text: &mut TextSystem,
        picture: Option<&mut PictureView>,
    ) -> bool {
        if fresh {
            text.forget_glyphs();
            if let Some(picture) = picture {
                picture.uploaded = None;
            }
        }
        fresh
    }

    pub(in crate::app::running) fn sync_graphics(&mut self) {
        if Self::sync_fresh(
            self.graphics.take_fresh(),
            &mut self.text,
            self.picture.as_mut(),
        ) {
            self.changed();
        }
        self.desk.power.tick(Instant::now());
        self.compact_rested();
        self.restart_if_card_lost();
    }

    pub(in crate::app::running) fn sync_rest(&mut self) {
        let shown: HashSet<SessionId> = self
            .frame
            .panes
            .iter()
            .filter_map(|pane| pane.active)
            .collect();
        let tabs = self
            .desk
            .sessions
            .map()
            .iter()
            .filter_map(|(&id, session)| {
                let shell = session.shell_id()?;
                Some(Shown {
                    id,
                    shell,
                    shown: shown.contains(&id),
                })
            });
        self.desk.power.set_tabs(tabs);
    }

    fn compact_rested(&mut self) {
        for id in self.desk.power.take_rested() {
            if let Some(session) = self.desk.sessions.get(id) {
                session.terminal().compact();
            }
        }
    }

    pub(in crate::app::running) fn clear_color(&self) -> Rgba {
        self.desk.theme.chrome
    }
}
