pub mod colors;
pub mod drag;
pub mod editing;
mod layout;
mod lifecycle;
pub mod press;
mod queries;
pub mod settings;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use trame::desk::{Desk, DeskSetup, FolderTarget};
use trame::geometry::Point;
use trame::input::{Keymap, Overrides, TextField};
use trame::render::Scene;
use trame::session::SessionId;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::ui::frame::SettingControl;
use trame::ui::{FrameInput, FrameLayout, FrameView, Hit, Motion, ResizeEdge, UiMetrics};
use trame::workspace::InstanceId;
use winit::keyboard::ModifiersState;

use drag::Drag;
use editing::{Editing, Menu};
use settings::SettingsState;

const DEFAULT_CELL: (f32, f32) = (8.0, 17.0);
type InitGeom = ((f32, f32), f32, UiMetrics);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowAction {
    Move,
    Resize(ResizeEdge),
    Minimize,
    ToggleMaximize,
    Raise,
}

#[derive(Clone, Debug)]
pub enum PickerJob {
    Folder(FolderTarget, Option<PathBuf>),
    Image(InstanceId, PathBuf),
}

pub struct WindowState {
    pub desk: Desk,
    pub text: TextSystem,
    pub chrome_scene: Scene,
    pub overlay_scene: Scene,
    pub metrics: UiMetrics,
    pub window_scale: f32,
    pub window_size: (f32, f32),
    pub maximized: bool,
    pub fullscreen: bool,
    pub hidden: bool,
    pub cell_size: (f32, f32),
    pub(super) keymap: Option<(Overrides, Keymap)>,
    pub(super) labels: HashMap<SessionId, String>,
    pub frame: FrameLayout,
    pub motion: Motion,
    pub view: FrameView,
    pub pointer: Option<Point>,
    pub pressed_hit: Option<Hit>,
    pub(super) last_click: Option<(Instant, Hit)>,
    pub wheel: f32,
    pub cursor: &'static str,
    pub drag: Option<Drag>,
    pub editing: Option<Editing>,
    pub setting_field: Option<(SettingControl, TextField)>,
    pub menu: Option<Menu>,
    pub settings: Option<SettingsState>,
    pub modifiers: ModifiersState,
    pub picture_open: bool,
    pub loading: bool,
    pub dirty: bool,
    pub pending_action: Option<WindowAction>,
    pub picker_req: Option<PickerJob>,
}

impl WindowState {
    pub fn new(setup: DeskSetup, size: (f32, f32), scale: f32) -> Result<Self, String> {
        let font_px = setup.preferences.font_pixels(scale);
        let mut text = TextSystem::new(font_px, DEFAULT_LINE_HEIGHT)
            .map_err(|e| format!("fonts unavailable: {e}"))?;
        let metrics = setup.preferences.ui_metrics(scale, 0.0);
        let desk = Desk::new(setup);
        let mut motion = Motion::default();
        let frame = initial_frame(&mut text, &desk, size, metrics, &mut motion);
        Ok(Self::build(desk, text, frame, motion, (size, scale, metrics)))
    }

    fn build(desk: Desk, text: TextSystem, frame: FrameLayout, motion: Motion, g: InitGeom) -> Self {
        Self {
            desk,
            text,
            chrome_scene: Scene::new(g.0.0, g.0.1),
            overlay_scene: Scene::new(g.0.0, g.0.1),
            metrics: g.2,
            window_scale: g.1,
            window_size: g.0,
            maximized: false,
            fullscreen: false,
            hidden: false,
            cell_size: DEFAULT_CELL,
            keymap: None,
            labels: HashMap::new(),
            frame,
            motion,
            view: FrameView::focused(),
            pointer: None,
            pressed_hit: None,
            last_click: None,
            wheel: 0.0,
            cursor: "default",
            drag: None,
            editing: None,
            setting_field: None,
            menu: None,
            settings: None,
            modifiers: ModifiersState::empty(),
            picture_open: false,
            loading: false,
            dirty: true,
            pending_action: None,
            picker_req: None,
        }
    }
}

fn initial_frame(
    text: &mut TextSystem,
    desk: &Desk,
    size: (f32, f32),
    metrics: UiMetrics,
    motion: &mut Motion,
) -> FrameLayout {
    let title = |_: SessionId| String::new();
    let input = FrameInput {
        workspace: &desk.workspace,
        width: size.0,
        height: size.1,
        metrics,
        maximized: false,
        fullscreen: false,
        title: &title,
        folder: None,
        update: None,
        unsaved: !desk.saver.keeps_session(),
        update_failed: false,
        strings: desk.strings,
    };
    let frame = FrameLayout::compute_with(text, &input, motion);
    motion.finish_layout();
    frame
}
