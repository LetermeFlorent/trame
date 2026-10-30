mod editing;
mod frame;
mod keys;
mod lifecycle;
mod pointer;
mod settings;
mod tabs;

use std::collections::HashMap;
use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::ModifiersState;
use winit::window::Window;

use trame::files::IconStore;
use trame::geometry::Point;
use trame::locale::Strings;
use trame::preferences::Preferences;
use trame::render::Scene;
use trame::session::{SessionEvent, SessionId, Transcripts};
use trame::text::TextSystem;
use trame::theme::Theme;
use trame::ui::{FrameLayout, FrameView, Motion, UiMetrics};
use trame::workspace::Workspace;

use super::graphics::Graphics;
use super::notice::Notifier;
use super::persist::Saver;
use super::power::Power;
use super::sessions::Sessions;
use super::visibility::Visibility;
use super::{UserEvent, window};
use editing::{Editing, Menu};
use frame::Loader;
pub use lifecycle::Offer;
use pointer::{Drag, Pointer, Selecting};
use settings::{SettingField, SettingsState};
use tabs::Closing;
pub use tabs::FolderTarget;

pub struct Running {
    window: Arc<Window>,
    graphics: Graphics,
    text: TextSystem,
    scene: Scene,
    theme: Theme,
    metrics: UiMetrics,
    window_scale: f32,
    preferences: Preferences,
    workspace: Workspace,
    sessions: Sessions,
    proxy: EventLoopProxy<UserEvent>,
    saver: Saver,
    transcripts: Transcripts,
    icons: IconStore,
    labels: HashMap<SessionId, String>,
    frame: FrameLayout,
    motion: Motion,
    view: FrameView,
    pointer: Pointer,
    drag: Option<Drag>,
    editing: Option<Editing>,
    setting_field: Option<SettingField>,
    menu: Option<Menu>,
    settings: Option<SettingsState>,
    selecting: Selecting,
    modifiers: ModifiersState,
    visibility: Visibility,
    loader: Option<Loader>,
    update_offer: Option<Offer>,
    restarting: bool,
    power: Power,
    notifier: Notifier,
    closing: Option<Closing>,
    strings: &'static Strings,
}

impl Running {
    pub fn session_event(&mut self, event_loop: &ActiveEventLoop, event: SessionEvent) {
        match event {
            SessionEvent::Output(id) => {
                self.power.output(id);
                self.output(id);
            }
            SessionEvent::Exited(id) => self.close_tab(event_loop, id),
        }
    }

    pub fn window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        if !matches!(event, WindowEvent::RedrawRequested) {
            self.power.touch();
        }
        match event {
            WindowEvent::CloseRequested => self.ask_close(event_loop, Closing::Window),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::Occluded(occluded) => self.visibility.set_occluded(occluded),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.rescale(scale_factor as f32)
            }
            WindowEvent::ThemeChanged(theme) => self.set_appearance(window::appearance_of(theme)),
            WindowEvent::Focused(focused) => {
                self.visibility.forget();
                self.focus_changed(focused);
                self.sync_background();
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => self.key(event_loop, &event),
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_moved(event_loop, Point::new(position.x as f32, position.y as f32))
            }
            WindowEvent::CursorLeft { .. } => self.pointer_left(),
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_button(event_loop, state, button)
            }
            WindowEvent::MouseWheel { delta, .. } => self.wheel(delta),
            WindowEvent::DroppedFile(path) => self.file_dropped(&path),
            _ => {}
        }
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        self.visibility.forget();
        self.graphics.resize(size.width, size.height);
        self.sync_background();
        self.changed();
    }

    fn sync_background(&mut self) {
        let minimized = self.window.is_minimized() == Some(true);
        self.power
            .set_background(minimized || !self.view.window_focused);
    }

    fn changed(&mut self) {
        self.relayout();
        self.window.request_redraw();
    }
}
