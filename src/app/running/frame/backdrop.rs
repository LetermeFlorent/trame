use std::time::Instant;

use winit::event_loop::ActiveEventLoop;

use trame::preferences::Preferences;
use trame::theme::Rgba;

use crate::app::running::Running;
use crate::app::window;

const SEE_THROUGH: Rgba = Rgba::new(0.0, 0.0, 0.0, 0.0);

impl Running {
    pub(in crate::app::running) fn sync_graphics(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.needs_reopen() {
            self.text.forget_glyphs();
            let window = self.window.clone();
            let atlas = self.text.atlas_mut();
            let translucent = self.preferences.acrylic;
            if let Err(error) = self.graphics.reopen(event_loop, window, atlas, translucent) {
                eprintln!("trame: the graphics card could not draw again: {error}");
            }
        }
        if self.graphics.take_fresh() {
            self.refresh_backdrop();
            self.changed();
        }
        self.idle.tick(Instant::now());
        self.restart_if_card_lost(event_loop);
    }

    pub(in crate::app::running) fn refresh_backdrop(&mut self) {
        let wanted = self.preferences.acrylic && self.graphics.is_translucent();
        let shown = (wanted || self.theme.is_translucent())
            && window::set_backdrop(&self.window, wanted, self.theme.appearance);
        self.theme.set_translucent_panel(shown);
    }

    pub(in crate::app::running) fn clear_color(&self) -> Rgba {
        match self.theme.is_translucent() {
            true => SEE_THROUGH,
            false => self.theme.chrome,
        }
    }

    pub(in crate::app::running) fn apply_system(&mut self, previous: &Preferences) {
        let current = &self.preferences;
        self.workspace.set_numbering(current.numbering);
        if previous.efficiency != current.efficiency {
            self.idle.set_enabled(current.efficiency);
        }
        if previous.acrylic != current.acrylic {
            self.graphics.want_translucent(current.acrylic);
            self.window.request_redraw();
        }
    }
}
