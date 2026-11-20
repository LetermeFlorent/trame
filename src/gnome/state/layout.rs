use std::time::Instant;

use trame::desk::{Effect, Effects};
use trame::preferences::Preferences;
use trame::session::SessionId;
use trame::text::DEFAULT_LINE_HEIGHT;
use trame::theme::{Appearance, Emblem, Theme};
use trame::ui::{FrameInput, FrameLayout};
use trame::workspace::Instance;

use super::{WindowAction, WindowState};

impl WindowState {
    pub fn relayout(&mut self) {
        if self.window_size.0 < 1.0 || self.window_size.1 < 1.0 {
            return;
        }
        self.prepare_relayout();
        self.compute_frame();
        self.mark_sessions();
        self.frame.blocked = self.loading;
        self.place_editor();
        self.place_menu();
        self.place_settings();
        self.place_message();
        let grids = self.grids();
        self.desk.resize_shells(&grids);
        self.sync_rest();
        self.refresh_hover();
    }

    fn prepare_relayout(&mut self) {
        self.text.receive_fonts();
        for inst in &self.desk.workspace.instances {
            if let Emblem::Image(id) = inst.emblem {
                self.desk.icons.load(id);
            }
        }
        let visible = self.desk.workspace.active_instance().map(Instance::tabs);
        self.labels = visible
            .unwrap_or_default()
            .into_iter()
            .map(|id| (id, self.desk.sessions.label(id)))
            .collect();
        self.motion.tick(Instant::now());
    }

    fn compute_frame(&mut self) {
        let folder = self.desk.active_folder();
        let update = self
            .desk
            .update
            .offer
            .as_ref()
            .map(|o| o.label(self.desk.strings));
        let labels = &self.labels;
        let ws = &self.desk.workspace;
        let sessions = &self.desk.sessions;
        let title = |id: SessionId| match ws.tab_name(id) {
            Some(n) => n.to_owned(),
            None => labels
                .get(&id)
                .cloned()
                .unwrap_or_else(|| sessions.label(id)),
        };
        let input = FrameInput {
            workspace: ws,
            width: self.window_size.0,
            height: self.window_size.1,
            metrics: self.metrics,
            maximized: self.maximized,
            fullscreen: self.fullscreen,
            title: &title,
            folder: folder.as_deref(),
            update: update.as_deref(),
            unsaved: !self.desk.saver.keeps_session(),
            update_failed: self.desk.update.failed,
            strings: self.desk.strings,
        };
        self.frame = FrameLayout::compute_with(&mut self.text, &input, &mut self.motion);
        self.motion.finish_layout();
    }

    pub fn apply(&mut self, effects: Effects) {
        for effect in effects {
            self.apply_one(effect);
        }
    }

    fn apply_one(&mut self, effect: Effect) {
        match effect {
            Effect::Redraw => self.dirty = true,
            Effect::Relayout => self.relayout(),
            Effect::Changed => self.changed(),
            Effect::Rescale => self.apply_scale(),
            Effect::Animate => self.motion.animate_next(),
            Effect::Raise => self.pending_action = Some(WindowAction::Raise),
            Effect::StartQueued => {
                let g = self.grids();
                self.desk.start_queued(&g);
            }
            Effect::StartShown => {
                let g = self.grids();
                if self.desk.start_shown(&g) {
                    self.relayout();
                }
            }
            Effect::Typed(id) => {
                if self.motion.scroll_offset(id) != 0.0 {
                    self.dirty = true;
                }
                self.motion.stop_scroll(id);
            }
            Effect::HostsChanged => self.reload_hosts(),
        }
    }

    pub fn apply_scale(&mut self) {
        self.metrics = self.desk.preferences.ui_metrics(self.window_scale, 0.0);
        let font = self.desk.preferences.font_pixels(self.window_scale);
        self.text.set_terminal_font(font, DEFAULT_LINE_HEIGHT);
        self.changed();
    }

    pub fn set_preferences(&mut self, preferences: Preferences) {
        let effects = self.desk.store_preferences(preferences);
        self.apply(effects);
    }

    pub fn set_appearance(&mut self, appearance: Appearance) {
        if appearance != self.desk.theme.appearance {
            self.desk.theme = Theme::new(appearance);
            self.desk
                .sessions
                .set_colors(self.desk.theme.reported_colors());
            self.dirty = true;
        }
    }
}
