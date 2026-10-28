mod first_frame;
mod saved;
mod surface;
mod text;
mod update;

use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::keyboard::ModifiersState;

use trame::files::IconStore;
use trame::preferences::{Preferences, Recall};
use trame::render::Scene;
use trame::session::Transcripts;
use trame::theme::Theme;
use trame::ui::FrameView;
use trame::workspace::{Snapshot, Workspace};

use super::tabs::Placement;
use super::{Pointer, Running, Selecting};
use crate::app::notice::Notifier;
use crate::app::persist::{self, Role, Saver};
use crate::app::power::Power;
use crate::app::sessions::Sessions;
use crate::app::update::UpdateStatus;
use crate::app::visibility::Visibility;
use crate::app::{UserEvent, window};

use first_frame::first_frame;
use surface::Surface;
pub use update::Offer;

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
        let restored = running.saver.owns() && running.restore_session(saved.as_ref());
        if !restored {
            running.open(Placement::Instance)?;
        }
        if !running.preferences.recalls(Recall::Text) {
            running.transcripts.clear();
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
        saved: Option<Snapshot>,
        role: Role,
    ) -> Result<Opened, Box<dyn Error>> {
        let window = saved
            .as_ref()
            .and_then(|snapshot| snapshot.window)
            .filter(|_| role.owns());
        let preferences = saved
            .as_ref()
            .map_or_else(Preferences::default, |snapshot| {
                snapshot.preferences.clone().sanitized()
            });
        let surface = Surface::open(event_loop, window, preferences)?;
        let mut running = Self::with_surface(surface, proxy, Saver::new(role));
        running.sessions.follow_preferences(&running.preferences);
        running.saver.remember_window(window);
        Ok(Opened { running, saved })
    }

    pub fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.sync_graphics(event_loop);
        let save = self.save_deadline();
        let text = self.transcript_deadline();
        let retry = self.visibility.retry(&self.window);
        let loader = self.loader_wake(Instant::now());
        let secret = self.sessions.secret_deadline(Instant::now());
        let wake = save
            .into_iter()
            .chain(text)
            .chain(retry)
            .chain(loader)
            .chain(secret)
            .chain(self.power.deadline())
            .min();
        event_loop.set_control_flow(wake.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }

    fn with_surface(mut surface: Surface, proxy: EventLoopProxy<UserEvent>, saver: Saver) -> Self {
        let metrics = surface.metrics();
        let workspace = Workspace::numbered(surface.preferences.numbering);
        let (frame, motion, strings) = first_frame(&mut surface, &workspace, metrics, &saver);
        Self {
            theme: Theme::new(window::appearance(&surface.window)),
            power: Power::new(surface.preferences.power_switches()),
            notifier: Notifier::default(),
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
            transcripts: Transcripts::new(saver.transcript_dir()),
            saver,
            icons: IconStore::new(persist::icon_dir()),
            labels: HashMap::new(),
            frame,
            motion,
            view: FrameView::focused(),
            pointer: Pointer::default(),
            drag: None,
            editing: None,
            setting_field: None,
            menu: None,
            settings: None,
            selecting: Selecting::default(),
            modifiers: ModifiersState::empty(),
            visibility: Visibility::default(),
            loader: None,
            update_offer: None,
            restarting: false,
            closing: None,
            strings,
        }
    }
}
