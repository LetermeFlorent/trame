mod graphics;
mod handler;
mod notice;
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
use std::path::{Path, PathBuf};

use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};

use trame::preferences::Updates;
use trame::session::SessionEvent;
use trame::workspace::{InstanceId, Snapshot};

use persist::Role;
use running::{FolderTarget, Offer, Running};
use update::UpdateStatus;

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
    let mut role = persist::claim();
    let saved = persist::load(&mut role);
    let updates = saved
        .as_ref()
        .map_or_else(Updates::default, |snapshot| snapshot.preferences.updates);
    let update = update::start(updates, move |status| {
        let _ = notifier.send_event(UserEvent::Update(status));
    });
    let mut app = App {
        proxy,
        update,
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
    role: Role,
    saved: Option<Snapshot>,
    running: Option<Running>,
    failure: Option<Box<dyn Error>>,
}

impl App {
    fn launch(&mut self, event_loop: &ActiveEventLoop) -> Result<Running, Box<dyn Error>> {
        let saved = self.saved.take();
        let role = mem::replace(&mut self.role, Role::Guest);
        Running::open_window(event_loop, self.proxy.clone(), saved, role)?.show(&self.update)
    }

    fn relaunch(&mut self, event_loop: &ActiveEventLoop, exe: &Path) -> io::Result<()> {
        match &mut self.running {
            Some(running) => running.restart_with(event_loop, || update::relaunch(exe)),
            None => {
                self.role.hand_over(|| update::relaunch(exe))?;
                event_loop.exit();
                Ok(())
            }
        }
    }

    fn updated(&mut self, event_loop: &ActiveEventLoop, status: UpdateStatus) {
        let status = match status {
            UpdateStatus::Installed(exe) => match self.relaunch(event_loop, &exe) {
                Ok(()) => return,
                Err(error) => {
                    eprintln!("trame: the new version did not start: {error}");
                    UpdateStatus::Done
                }
            },
            UpdateStatus::Available(version) => {
                return self.offer(Offer {
                    version,
                    ready: None,
                });
            }
            UpdateStatus::Ready { version, exe } => {
                return self.offer(Offer {
                    version,
                    ready: Some(exe),
                });
            }
            status => status,
        };
        if let Some(running) = &mut self.running {
            running.set_update(&status);
        }
        self.update = status;
    }

    fn offer(&mut self, offer: Offer) {
        if let Some(running) = &mut self.running {
            running.offer_update(offer);
        }
    }
}
