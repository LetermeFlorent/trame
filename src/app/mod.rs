mod graphics;
mod handler;
mod notice;
mod pending;
mod persist;
mod power;
mod running;
mod sessions;
mod update;
mod visibility;
mod window;

use std::error::Error;
use std::io;
use std::path::PathBuf;

use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};

use trame::session::SessionEvent;
use trame::workspace::{InstanceId, Snapshot};

use pending::PendingOffer;
use running::{FolderTarget, Offer, Running};
use update::{Handoff, UpdateStatus};

#[derive(Clone, Debug)]
pub enum UserEvent {
    Session(SessionEvent),
    Update(UpdateStatus),
    Folder(FolderTarget, PathBuf),
    Icon(InstanceId, u64),
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let notifier = proxy.clone();
    let saved = persist::load();
    let update = update::start(move |status| {
        let _ = notifier.send_event(UserEvent::Update(status));
    });
    let mut app = App {
        proxy,
        update,
        offer: PendingOffer::default(),
        saved,
        running: None,
        failure: None,
    };
    event_loop.run_app(&mut app)?;
    app.failure.map_or(Ok(()), Err)
}

struct App {
    proxy: EventLoopProxy<UserEvent>,
    update: UpdateStatus,
    offer: PendingOffer,
    saved: Option<Snapshot>,
    running: Option<Running>,
    failure: Option<Box<dyn Error>>,
}

impl App {
    fn launch(&mut self, event_loop: &ActiveEventLoop) -> Result<Running, Box<dyn Error>> {
        let saved = self.saved.take();
        let opened = Running::open_window(event_loop, self.proxy.clone(), saved)?;
        let mut running = opened.show(&self.update)?;
        self.offer.opened(&mut running);
        Ok(running)
    }

    fn hand_over(&mut self, handoff: &Handoff) -> io::Result<()> {
        if let Some(running) = &mut self.running {
            running.save_for_restart();
        }
        handoff.start()
    }

    fn updated(&mut self, event_loop: &ActiveEventLoop, status: UpdateStatus) {
        let status = match status {
            UpdateStatus::Installed(handoff) => match self.hand_over(&handoff) {
                Ok(()) => {
                    self.running = None;
                    event_loop.exit();
                    return;
                }
                Err(error) => {
                    eprintln!("trame: the new version did not start: {error}");
                    UpdateStatus::Done
                }
            },
            UpdateStatus::Available(version) => {
                let offer = Offer { version };
                return self.offer.send(offer, self.running.as_mut());
            }
            status => status,
        };
        if let Some(running) = &mut self.running {
            running.set_update(&status);
        }
        self.update = status;
    }
}
