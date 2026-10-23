mod loader;
mod paint;
mod style;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use winit::window::Window;

use trame::locale::Strings;
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::ui::{FrameInput, FrameLayout, Motion, UiMetrics};
use trame::workspace::{Instance, Workspace};

use super::Running;
use crate::app::sessions::Sessions;
use crate::app::window;
pub(super) use loader::Loader;

pub(super) struct LayoutInput<'a> {
    pub(super) workspace: &'a Workspace,
    pub(super) window: &'a Window,
    pub(super) metrics: UiMetrics,
    pub(super) folder: Option<&'a Path>,
    pub(super) update: Option<&'a str>,
    pub(super) strings: &'static Strings,
}

pub(super) fn compute(
    text: &mut TextSystem,
    layout: &LayoutInput,
    motion: &mut Motion,
    title: &dyn Fn(SessionId) -> String,
) -> FrameLayout {
    let size = layout.window.inner_size();
    let input = FrameInput {
        workspace: layout.workspace,
        width: size.width as f32,
        height: size.height as f32,
        metrics: layout.metrics,
        maximized: layout.window.is_maximized(),
        title,
        folder: layout.folder,
        update: layout.update,
        strings: layout.strings,
    };
    let frame = FrameLayout::compute_with(text, &input, motion);
    motion.finish_layout();
    frame
}

fn title_of(
    workspace: &Workspace,
    labels: &HashMap<SessionId, String>,
    sessions: &Sessions,
    id: SessionId,
) -> String {
    match workspace.tab_name(id) {
        Some(name) => name.to_owned(),
        None => labels
            .get(&id)
            .cloned()
            .unwrap_or_else(|| sessions.label(id)),
    }
}

impl Running {
    pub(super) fn relayout(&mut self) {
        if !window::has_area(&self.window) {
            return;
        }
        let visible = self.workspace.active_instance().map(Instance::tabs);
        self.labels = visible
            .unwrap_or_default()
            .into_iter()
            .map(|id| (id, self.sessions.label(id)))
            .collect();
        let title = |id: SessionId| title_of(&self.workspace, &self.labels, &self.sessions, id);
        let folder = self.active_folder();
        let layout = LayoutInput {
            workspace: &self.workspace,
            window: &self.window,
            metrics: self.metrics,
            folder: folder.as_deref(),
            update: self.update_offer.as_deref(),
            strings: self.strings,
        };
        self.motion.tick(Instant::now());
        self.frame = compute(&mut self.text, &layout, &mut self.motion, &title);
        self.frame.blocked = self.loader.is_some();
        self.place_editor();
        self.place_menu();
        self.place_settings();
        self.place_confirm();
        self.sync_sizes();
        self.refresh_hover();
    }

    pub(super) fn active_folder(&self) -> Option<PathBuf> {
        let tab = self.workspace.active_tab()?;
        self.sessions.cwd(tab)
    }

    pub(super) fn tab_title(&self, id: SessionId) -> String {
        title_of(&self.workspace, &self.labels, &self.sessions, id)
    }

    fn sync_sizes(&mut self) {
        let cell = self.text.cell_metrics();
        for pane in &self.frame.panes {
            let size = pane.grid_size(&cell);
            for tab in &pane.tabs {
                self.sessions.resize(*tab, size);
            }
        }
    }
}
