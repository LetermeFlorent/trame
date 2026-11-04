mod graphics;
mod handler;
mod notice;
mod pending;
mod persist;
mod power;
mod running;
#[cfg(test)]
mod scratch;
mod sessions;
mod update;
mod visibility;
mod window;

use std::error::Error;
use std::io;
use std::mem;
use std::path::PathBuf;

use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};

use trame::log_error;
use trame::session::SessionEvent;
use trame::workspace::{InstanceId, Snapshot};

use pending::PendingOffer;
use persist::Role;
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
    if let Some(path) = persist::log_path() {
        trame::files::start_log(path);
    }
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let notifier = proxy.clone();
    let mut role = persist::claim();
    let saved = persist::load(&mut role);
    let update = update::start(move |status| {
        let _ = notifier.send_event(UserEvent::Update(status));
    });
    let mut app = App {
        proxy,
        update,
        offer: PendingOffer::default(),
        role,
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
    role: Role,
    saved: Option<Snapshot>,
    running: Option<Running>,
    failure: Option<Box<dyn Error>>,
}

impl App {
    fn launch(&mut self, event_loop: &ActiveEventLoop) -> Result<Running, Box<dyn Error>> {
        let saved = self.saved.take();
        let role = mem::replace(&mut self.role, Role::Guest);
        let opened = Running::open_window(event_loop, self.proxy.clone(), saved, role)?;
        let mut running = opened.show(&self.update)?;
        self.offer.opened(&mut running);
        Ok(running)
    }

    fn hand_over(&mut self, event_loop: &ActiveEventLoop, handoff: &Handoff) -> io::Result<()> {
        match &mut self.running {
            Some(running) => running.restart_with(event_loop, || handoff.start()),
            None => {
                self.role.hand_over(|| handoff.start())?;
                event_loop.exit();
                Ok(())
            }
        }
    }

    fn updated(&mut self, event_loop: &ActiveEventLoop, status: UpdateStatus) {
        let status = match status {
            UpdateStatus::Installed(handoff) => match self.hand_over(event_loop, &handoff) {
                Ok(()) => return,
                Err(error) => {
                    log_error!("the new version did not start: {error}");
                    UpdateStatus::Failed
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
