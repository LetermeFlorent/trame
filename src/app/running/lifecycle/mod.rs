mod saved;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::keyboard::ModifiersState;
use winit::window::Window;

use trame::render::Scene;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::theme::Theme;
use trame::ui::metrics::TERMINAL_FONT_SIZE;
use trame::ui::{FrameView, Motion, UiMetrics};
use trame::workspace::{Snapshot, WindowSnapshot, Workspace};

use super::tabs::Placement;
use super::{Pointer, Running, Selecting, frame};
use crate::app::graphics::Graphics;
use crate::app::persist::{self, Saver};
use crate::app::sessions::Sessions;
use crate::app::visibility::Visibility;
use crate::app::{UserEvent, window};

struct Surface {
    window: Arc<Window>,
    graphics: Graphics,
    text: TextSystem,
    metrics: UiMetrics,
}

impl Surface {
    fn open(
        event_loop: &ActiveEventLoop,
        saved: Option<WindowSnapshot>,
    ) -> Result<Self, Box<dyn Error>> {
        let window = Arc::new(event_loop.create_window(window::restored_attributes(saved))?);
        window::round_corners(&window);
        let metrics = UiMetrics::new(window.scale_factor() as f32);
        let mut text = TextSystem::new(TERMINAL_FONT_SIZE * metrics.scale, DEFAULT_LINE_HEIGHT)?;
        let graphics = Graphics::new(event_loop, window.clone(), text.atlas_mut())?;
        Ok(Self {
            window,
            graphics,
            text,
            metrics,
        })
    }
}

pub struct Opened {
    running: Running,
    saved: Option<Snapshot>,
}

impl Opened {
    pub fn show(self) -> Result<Running, Box<dyn Error>> {
        let Opened { mut running, saved } = self;
        if !running.restore_session(saved.as_ref()) {
            running.open(Placement::Instance)?;
        }
        running.draw();
        running.window.set_visible(true);
        Ok(running)
    }
}

impl Running {
    pub fn open_window(
        event_loop: &ActiveEventLoop,
        proxy: EventLoopProxy<UserEvent>,
    ) -> Result<Opened, Box<dyn Error>> {
        let saved = persist::load();
        let window = saved.as_ref().and_then(|snapshot| snapshot.window);
        let surface = Surface::open(event_loop, window)?;
        let mut running = Self::with_surface(surface, proxy);
        running.saver.remember_window(window);
        Ok(Opened { running, saved })
    }

    pub fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let save = self.save_deadline();
        let retry = self.visibility.retry(&self.window);
        let wake = save.into_iter().chain(retry).min();
        event_loop.set_control_flow(wake.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }

    fn with_surface(surface: Surface, proxy: EventLoopProxy<UserEvent>) -> Self {
        let Surface {
            window,
            graphics,
            mut text,
            metrics,
        } = surface;
        let workspace = Workspace::new();
        let mut motion = Motion::default();
        let layout_input = frame::LayoutInput {
            workspace: &workspace,
            window: &window,
            metrics,
        };
        let frame = frame::compute(&mut text, &layout_input, &mut motion, &|_| String::new());
        Self {
            theme: Theme::new(window::appearance(&window)),
            window,
            graphics,
            text,
            scene: Scene::new(0.0, 0.0),
            metrics,
            font_size: TERMINAL_FONT_SIZE,
            workspace,
            sessions: Sessions::new(proxy),
            saver: Saver::new(),
            labels: HashMap::new(),
            frame,
            motion,
            view: FrameView {
                window_focused: true,
                ..FrameView::default()
            },
            pointer: Pointer::default(),
            drag: None,
            editing: None,
            selecting: Selecting::default(),
            modifiers: ModifiersState::empty(),
            visibility: Visibility::default(),
        }
    }
}
