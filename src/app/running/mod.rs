mod commands;
mod drag;
mod frame;
mod keyboard;
mod mouse;
mod scroll;
mod tabs;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::ModifiersState;
use winit::window::Window;

use trame::geometry::Point;
use trame::render::Scene;
use trame::session::{SessionEvent, SessionId};
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::theme::Theme;
use trame::ui::metrics::TERMINAL_FONT_SIZE;
use trame::ui::{FrameLayout, FrameView, UiMetrics};
use trame::workspace::Workspace;

use super::graphics::Graphics;
use super::sessions::Sessions;
use super::{UserEvent, window};
use drag::Drag;
use mouse::Pointer;
use tabs::Placement;

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
    labels: HashMap<SessionId, String>,
    frame: FrameLayout,
    view: FrameView,
    pointer: Pointer,
    drag: Option<Drag>,
    modifiers: ModifiersState,
}

impl Running {
    pub fn start(
        event_loop: &ActiveEventLoop,
        proxy: EventLoopProxy<UserEvent>,
    ) -> Result<Self, Box<dyn Error>> {
        let window = Arc::new(event_loop.create_window(window::attributes())?);
        window::round_corners(&window);
        let metrics = UiMetrics::new(window.scale_factor() as f32);
        let mut text = TextSystem::new(TERMINAL_FONT_SIZE * metrics.scale, DEFAULT_LINE_HEIGHT)?;
        let graphics = Graphics::new(event_loop, window.clone(), text.atlas_mut())?;
        let theme = Theme::new(window::appearance(&window));
        let workspace = Workspace::new();
        let frame = frame::compute(&mut text, &workspace, &window, metrics, &|_| String::new());
        let mut running = Self {
            window,
            graphics,
            text,
            scene: Scene::new(0.0, 0.0),
            theme,
            metrics,
            font_size: TERMINAL_FONT_SIZE,
            workspace,
            sessions: Sessions::new(proxy),
            labels: HashMap::new(),
            frame,
            view: FrameView {
                window_focused: true,
                ..FrameView::default()
            },
            pointer: Pointer::default(),
            drag: None,
            modifiers: ModifiersState::empty(),
        };
        running.open(Placement::Instance)?;
        running.redraw();
        running.window.set_visible(true);
        Ok(running)
    }

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
