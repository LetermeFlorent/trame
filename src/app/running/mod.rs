mod editing;
mod effects;
mod frame;
mod keys;
mod lifecycle;
mod pointer;
mod settings;
mod tabs;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::ModifiersState;
use winit::window::Window;

use trame::desk::{Closing, Desk};
use trame::geometry::Point;
use trame::input::{Keymap, Overrides};
use trame::render::Scene;
use trame::session::{SessionEvent, SessionId};
use trame::text::TextSystem;
use trame::ui::terminal::links::LinkCache;
use trame::ui::{FrameLayout, FrameView, Motion, UiMetrics};

use super::graphics::Graphics;
use super::notice::Notifier;
use super::visibility::Visibility;
use super::{AppEvent, window};
use editing::{Editing, Menu};
use frame::Loader;
use keys::Memo;
use pointer::{Drag, Pointer, Selecting};
use settings::{SettingField, SettingsState};
pub use tabs::Lifted;

pub struct Running {
    desk: Desk,
    window: Arc<Window>,
    graphics: Graphics,
    text: TextSystem,
    scene: Scene,
    metrics: UiMetrics,
    window_scale: f32,
    keymap: Memo<Overrides, Keymap>,
    links: RefCell<LinkCache>,
    proxy: EventLoopProxy<AppEvent>,
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
    notifier: Notifier,
}

impl Running {
    pub fn session_event(&mut self, exiting: bool, event: SessionEvent) {
        match event {
            SessionEvent::Output(id) => self.output(id),
            SessionEvent::Exited(id) => {
                let effects = self.desk.shell_exited(exiting, id);
                self.apply(effects);
            }
        }
    }

    pub fn window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        if !matches!(event, WindowEvent::RedrawRequested) {
            self.desk.power.touch();
        }
        match event {
            WindowEvent::CloseRequested => self.ask_close(Closing::Window),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::Occluded(occluded) => self.visibility.set_occluded(occluded),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.rescale(scale_factor as f32)
            }
            WindowEvent::ThemeChanged(theme) => {
                let effects = self.desk.set_appearance(window::appearance_of(theme));
                self.apply(effects);
            }
            WindowEvent::Focused(focused) => {
                self.visibility.forget();
                self.focus_changed(focused);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
                self.refresh_hover();
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => self.key(event_loop.exiting(), &event),
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_moved(event_loop, Point::new(position.x as f32, position.y as f32))
            }
            WindowEvent::CursorLeft { .. } => self.pointer_left(),
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_button(event_loop.exiting(), state, button)
            }
            WindowEvent::MouseWheel { delta, .. } => self.wheel(delta),
            WindowEvent::DroppedFile(path) => self.file_dropped(&path),
            _ => {}
        }
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        self.visibility.forget();
        self.graphics.resize(size.width, size.height);
        self.changed();
    }

    fn changed(&mut self) {
        self.relayout();
        self.window.request_redraw();
    }
}
