mod commands;
mod editing;
mod frame;
mod keyboard;
mod lifecycle;
mod pointer;
mod tabs;

use std::collections::HashMap;
use std::sync::Arc;

use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
use winit::window::Window;

use trame::geometry::Point;
use trame::render::Scene;
use trame::session::{SessionEvent, SessionId};
use trame::text::TextSystem;
use trame::theme::Theme;
use trame::ui::{FrameLayout, FrameView, Motion, UiMetrics};
use trame::workspace::Workspace;

use super::graphics::Graphics;
use super::persist::Saver;
use super::sessions::Sessions;
use super::{UserEvent, window};
use editing::Editing;
use pointer::{Drag, Pointer, Selecting};

pub struct Running {
    window: Arc<Window>,
    graphics: Graphics,
    text: TextSystem,
    scene: Scene,
    theme: Theme,
    metrics: UiMetrics,
    font_size: f32,
    workspace: Workspace,
    sessions: Sessions,
    saver: Saver,
    labels: HashMap<SessionId, String>,
    frame: FrameLayout,
    motion: Motion,
    view: FrameView,
    pointer: Pointer,
    drag: Option<Drag>,
    editing: Option<Editing>,
    selecting: Selecting,
    modifiers: ModifiersState,
}

impl Running {
    pub fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Session(SessionEvent::Output(id)) => self.output(id),
            UserEvent::Session(SessionEvent::Exited(id)) => self.close_tab(event_loop, id),
        }
    }

    pub fn window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::Resized(size) => {
                self.graphics.resize(size.width, size.height);
                self.changed();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.rescale(scale_factor as f32)
            }
            WindowEvent::ThemeChanged(theme) => self.set_appearance(window::appearance_of(theme)),
            WindowEvent::Focused(focused) => self.focus_changed(focused),
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => self.key(event_loop, &event),
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_moved(Point::new(position.x as f32, position.y as f32))
            }
            WindowEvent::CursorLeft { .. } => self.pointer_left(),
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_button(event_loop, state, button)
            }
            WindowEvent::MouseWheel { delta, .. } => self.wheel(delta),
            _ => {}
        }
    }

    fn changed(&mut self) {
        self.relayout();
        self.window.request_redraw();
    }
}
