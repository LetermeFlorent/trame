use trame::session::SessionId;
use trame::ui::{FrameSources, draw_frame};

use crate::app::running::Running;
use crate::app::window;

const ATLAS_ATTEMPTS: usize = 2;

impl Running {
    pub(in crate::app::running) fn redraw(&mut self) {
        if self.visibility.is_hidden(&self.window) {
            self.visibility.missed();
            return;
        }
        self.draw();
    }

    pub(in crate::app::running) fn draw(&mut self) {
        self.sessions.clear_output();
        self.relayout();
        if !window::has_area(&self.window) {
            return;
        }
        let size = self.window.inner_size();
        let drag = self.drag_view();
        self.motion.hover(self.view.hover);
        for _ in 0..ATLAS_ATTEMPTS {
            self.scene.reset(size.width as f32, size.height as f32);
            let sources = FrameSources {
                theme: &self.theme,
                frame: &self.frame,
                view: &self.view,
                sessions: self.sessions.map(),
                motion: &self.motion,
                drag: drag.as_ref(),
                loader: None,
            };
            draw_frame(&mut self.scene, &mut self.text, &sources);
            if !self.text.end_frame() {
                break;
            }
        }
        let clear = self.theme.chrome;
        let presented =
            self.graphics
                .present(&self.window, &self.scene, self.text.atlas_mut(), clear);
        if presented {
            self.visibility.drawn();
        } else {
            self.visibility.missed();
        }
        if self.motion.is_busy() {
            self.window.request_redraw();
        }
    }

    pub(in crate::app::running) fn output(&mut self, id: SessionId) {
        self.forward_clipboard(id);
        if self.frame.panes.iter().any(|pane| pane.active == Some(id)) {
            self.window.request_redraw();
            return;
        }
        let Some(session) = self.sessions.get(id) else {
            return;
        };
        session.take_output();
        if self
            .labels
            .get(&id)
            .is_some_and(|known| *known != session.label())
        {
            self.window.request_redraw();
        }
    }
}
