use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use trame::session::SessionId;
use trame::theme::Emblem;
use trame::ui::{FrameSources, draw_frame};

use crate::app::running::Running;
use crate::app::running::pointer::HoverPreview;
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
        let now = Instant::now();
        self.text.receive_fonts();
        self.text.receive_images();
        self.drop_gone_loader(now);
        self.relayout();
        if !window::has_area(&self.window) {
            return;
        }
        if self.start_shown() {
            self.relayout();
        }
        Self::sync_fresh(
            self.graphics.take_fresh(),
            &mut self.text,
            self.picture.as_mut(),
        );
        self.sync_picture();
        if self.picture_active() {
            self.refresh_picture_hover();
        }
        self.paint(now);
        let clear = self.clear_color();
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

    fn paint(&mut self, now: Instant) {
        self.load_icons();
        let size = self.window.inner_size();
        let drag = self.drag_view();
        let preview = HoverPreview::frame(self.picture.is_some(), self.pointer.preview.as_ref());
        let loader = self.loader.as_ref().map(|loader| loader.view(now));
        self.motion.hover(self.view.hover);
        let on = self.desk.preferences.detect_links;
        let ids = self.frame.panes.iter().filter_map(|p| p.active);
        let cwds = visible_cwds(ids, on, |id| self.desk.sessions.cwd(id));
        for _ in 0..ATLAS_ATTEMPTS {
            self.scene.reset(size.width as f32, size.height as f32);
            let sources = FrameSources {
                theme: &self.desk.theme,
                frame: &self.frame,
                view: &self.view,
                sessions: self.desk.sessions.map(),
                motion: &self.motion,
                drag: drag.as_ref(),
                loader: loader.as_ref(),
                icons: &self.desk.icons,
                links: self.desk.preferences.detect_links.then_some(&self.links),
                cwd: Some(&|id| cwds.get(&id).cloned().flatten()),
                picture: self.picture_frame(),
                preview,
            };
            draw_frame(&mut self.scene, &mut self.text, &sources);
            if !self.text.end_frame() {
                break;
            }
        }
    }

    fn load_icons(&mut self) {
        for instance in &self.desk.workspace.instances {
            if let Emblem::Image(id) = instance.emblem {
                self.desk.icons.load(id);
            }
        }
    }

    pub(in crate::app::running) fn output(&mut self, id: SessionId) {
        self.desk.sessions.output(id);
        if self.desk.sessions.notice_exit(id) {
            self.relayout();
        }
        self.forward_clipboard(id);
        self.forward_attention(id);
        if self.pane_showing(id).is_some() || self.relabeled(id) {
            self.window.request_redraw();
        }
    }

    fn relabeled(&self, id: SessionId) -> bool {
        let Some(session) = self.desk.sessions.get(id) else {
            return false;
        };
        let known = self.labels.get(&id);
        known.is_some_and(|known| *known != session.label())
    }
}

fn visible_cwds(
    ids: impl IntoIterator<Item = SessionId>,
    on: bool,
    mut resolve: impl FnMut(SessionId) -> Option<PathBuf>,
) -> HashMap<SessionId, Option<PathBuf>> {
    let mut cwds = HashMap::new();
    for id in ids.into_iter().filter(|_| on) {
        cwds.entry(id).or_insert_with(|| resolve(id));
    }
    cwds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_cwds_caches_once_when_enabled() {
        let (a, b) = (SessionId(1), SessionId(2));
        let mut n = 0;
        let cwds = visible_cwds([a, b, a], true, |id| {
            n += 1;
            (id == a).then(|| PathBuf::from("/work"))
        });
        assert_eq!(n, 2);
        assert!(cwds.len() == 2 && cwds[&a].is_some() && cwds[&b].is_none());
        assert!(visible_cwds([a], false, |_| unreachable!()).is_empty());
    }
}
