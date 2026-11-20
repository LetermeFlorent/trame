use std::collections::HashSet;

use trame::desk::Grids;
use trame::geometry::Point;
use trame::host::power::Shown;
use trame::input::Keymap;
use trame::session::SessionId;

use super::{WindowAction, WindowState};

impl WindowState {
    pub fn changed(&mut self) {
        self.relayout();
        self.dirty = true;
    }

    pub fn act(&mut self, action: WindowAction) {
        self.pending_action = Some(action);
    }

    pub fn drop_press(&mut self) {
        self.pressed_hit = None;
        self.view.pressed = None;
    }

    pub fn modal_open(&self) -> bool {
        self.frame.blocked
            || self.picture_open
            || self.desk.closing.is_some()
            || self.menu.is_some()
            || self.editing.is_some()
            || self.settings.is_some()
            || self.setting_field.is_some()
    }

    pub fn overlay_picks(&self, point: Point) -> bool {
        if self.frame.blocked
            || self.desk.closing.is_some()
            || self.menu.is_some()
            || self.editing.is_some()
            || self.settings.is_some()
            || self.drag.is_some()
        {
            return true;
        }
        self.frame
            .panes
            .iter()
            .filter_map(|p| p.offline.as_ref().map(|c| (p.body, c)))
            .any(|(body, _)| body.contains(point))
    }

    pub fn keymap(&mut self) -> &Keymap {
        let prefs = &self.desk.preferences.shortcuts;
        let platform = self.metrics.platform;
        let rebuild = self.keymap.as_ref().is_none_or(|(saved, _)| saved != prefs);
        if rebuild {
            self.keymap = Some((prefs.clone(), Keymap::new(platform, prefs)));
        }
        &self.keymap.as_ref().unwrap().1
    }

    pub fn tab_title(&self, id: SessionId) -> String {
        match self.desk.workspace.tab_name(id) {
            Some(name) => name.to_owned(),
            None => self
                .labels
                .get(&id)
                .cloned()
                .unwrap_or_else(|| self.desk.sessions.label(id)),
        }
    }

    pub fn grids(&self) -> Grids {
        let (cw, ch) = (self.cell_size.0.max(1.0), self.cell_size.1.max(1.0));
        let mut grids = Grids::default();
        for pane in &self.frame.panes {
            let rows = (pane.grid_area.height / ch).floor().max(1.0) as usize;
            let cols = (pane.grid_area.width / cw).floor().max(1.0) as usize;
            grids.add_pane(&pane.tabs, pane.active, (rows, cols));
        }
        grids
    }

    pub fn mark_sessions(&mut self) {
        let sessions = &self.desk.sessions;
        self.frame
            .mark_working(&self.desk.workspace, |id| sessions.working(id));
        self.frame
            .mark_offline(&mut self.text, self.desk.strings, |id| sessions.offline(id));
    }

    pub fn sync_rest(&mut self) {
        let shown: HashSet<SessionId> = self.frame.panes.iter().filter_map(|p| p.active).collect();
        let tabs = self.desk.sessions.map().iter().filter_map(|(&id, s)| {
            Some(Shown {
                id,
                shell: s.shell_id()?,
                shown: shown.contains(&id),
            })
        });
        self.desk.power.set_tabs(tabs);
    }
}
