mod first_frame;
mod opened;
mod peers;
mod quit;
mod saved;
mod surface;
mod update;

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::ModifiersState;

use trame::desk::{Desk, DeskSetup};
use trame::host::persist::{self, Slot};
use trame::preferences::Preferences;
use trame::render::Scene;
use trame::ui::FrameView;
use trame::workspace::Snapshot;

use super::{Memo, Pointer, Running, Selecting};
use crate::app::notice::Notifier;
use crate::app::visibility::Visibility;
use crate::app::{AppEvent, explorer, relay, window};

use first_frame::first_frame;
pub use opened::Opened;
use surface::Surface;

impl Running {
    pub fn open_window(
        event_loop: &ActiveEventLoop,
        proxy: EventLoopProxy<AppEvent>,
        saved: Option<Snapshot>,
        slot: Slot,
        shared: Option<Preferences>,
    ) -> Result<Opened, Box<dyn Error>> {
        let window = saved
            .as_ref()
            .and_then(|snapshot| snapshot.window)
            .filter(|_| slot.owns());
        let mut preferences = shared
            .or_else(|| saved.as_ref().map(|snapshot| snapshot.preferences.clone()))
            .unwrap_or_default()
            .sanitized();
        preferences.explorer_menu = explorer::menu_installed();
        let surface = Surface::open(event_loop, window, &preferences)?;
        let desk = Desk::new(DeskSetup {
            preferences,
            slot,
            notifier: relay::session_notifier(proxy.clone(), surface.window.id()),
            appearance: window::appearance(&surface.window),
            icon_dir: persist::icon_dir(),
        });
        let mut running = Self::with_surface(surface, proxy, desk);
        running.desk.saver.remember_window(window);
        Ok(Opened { running, saved })
    }

    pub fn about_to_wait(&mut self) -> Option<Instant> {
        self.sync_graphics();
        let retry = self.visibility.retry(&self.window);
        self.desk
            .power
            .set_background(self.visibility.seen_hidden());
        let now = Instant::now();
        let next = self
            .desk
            .next_deadline(now, || window::snapshot(&self.window));
        self.apply(next.effects);
        let loader = self.loader_wake(now);
        let work = self.work_wake(now);
        [next.value, retry, loader, work]
            .into_iter()
            .flatten()
            .min()
    }

    fn with_surface(mut surface: Surface, proxy: EventLoopProxy<AppEvent>, desk: Desk) -> Self {
        let first = first_frame(&mut surface, &desk);
        Self {
            desk,
            notifier: Notifier::default(),
            window: surface.window,
            graphics: surface.graphics,
            text: surface.text,
            scene: Scene::new(0.0, 0.0),
            metrics: first.metrics,
            window_scale: surface.window_scale,
            keymap: Memo::default(),
            links: RefCell::default(),
            proxy,
            labels: HashMap::new(),
            frame: first.frame,
            motion: first.motion,
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
        }
    }
}
