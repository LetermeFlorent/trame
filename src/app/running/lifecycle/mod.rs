mod saved;
mod surface;
mod text;
mod update;

use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::keyboard::ModifiersState;

use trame::locale::Strings;
use trame::preferences::Preferences;
use trame::render::Scene;
use trame::session::Transcripts;
use trame::theme::Theme;
use trame::ui::{FrameLayout, FrameView, Motion, UiMetrics};
use trame::workspace::{Snapshot, Workspace};

use super::tabs::Placement;
use super::{Pointer, Running, Selecting, frame};
use crate::app::persist::{self, Saver};
use crate::app::sessions::Sessions;
use crate::app::update::UpdateStatus;
use crate::app::visibility::Visibility;
use crate::app::{UserEvent, window};
use surface::Surface;

pub struct Opened {
    running: Running,
    saved: Option<Snapshot>,
}

impl Opened {
    pub fn show(self, update: &UpdateStatus) -> Result<Running, Box<dyn Error>> {
        let Opened { mut running, saved } = self;
        running.set_update(update);
        if running.loader.is_some() {
            running.draw();
            running.window.set_visible(true);
        }
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
        let preferences = saved
            .as_ref()
            .map_or_else(Preferences::default, |snapshot| {
                snapshot.preferences.clone().sanitized()
            });
        let surface = Surface::open(event_loop, window, preferences)?;
        let mut running = Self::with_surface(surface, proxy);
        running.saver.remember_window(window);
        if !running.preferences.restore_text {
            running.transcripts.clear();
        }
        Ok(Opened { running, saved })
    }

    pub fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let save = self.save_deadline();
        let text = self.transcript_deadline();
        let retry = self.visibility.retry(&self.window);
        let loader = self.loader_wake(Instant::now());
        let wake = save
            .into_iter()
            .chain(text)
            .chain(retry)
            .chain(loader)
            .min();
        event_loop.set_control_flow(wake.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }

    fn with_surface(mut surface: Surface, proxy: EventLoopProxy<UserEvent>) -> Self {
        let preferences = &surface.preferences;
        let metrics = UiMetrics::new(preferences.ui_scale(surface.window_scale))
            .with_frame_border(window::frame_border(&surface.window));
        let workspace = Workspace::new();
        let mut motion = Motion::default();
        let strings = preferences.language.resolve().strings();
        let frame = first_frame(&mut surface, &workspace, metrics, &mut motion, strings);
        Self {
            theme: Theme::new(window::appearance(&surface.window)),
            window: surface.window,
            graphics: surface.graphics,
            text: surface.text,
            scene: Scene::new(0.0, 0.0),
            metrics,
            window_scale: surface.window_scale,
            preferences: surface.preferences,
            workspace,
            sessions: Sessions::new(proxy.clone()),
            proxy,
            saver: Saver::new(),
            transcripts: Transcripts::new(persist::transcript_dir()),
            labels: HashMap::new(),
            frame,
            motion,
            view: FrameView::focused(),
            pointer: Pointer::default(),
            drag: None,
            editing: None,
            command_field: None,
            menu: None,
            settings_open: false,
            selecting: Selecting::default(),
            modifiers: ModifiersState::empty(),
            visibility: Visibility::default(),
            loader: None,
            update_offer: None,
            strings,
        }
    }
}

fn first_frame(
    surface: &mut Surface,
    workspace: &Workspace,
    metrics: UiMetrics,
    motion: &mut Motion,
    strings: &'static Strings,
) -> FrameLayout {
    let layout_input = frame::LayoutInput {
        workspace,
        window: &surface.window,
        metrics,
        folder: None,
        update: None,
        strings,
    };
    frame::compute(&mut surface.text, &layout_input, motion, &|_| String::new())
}
