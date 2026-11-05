mod door;
mod fleet;
mod graphics;
mod handler;
mod notice;
mod pending;
mod persist;
mod power;
mod restart;
mod running;
#[cfg(test)]
mod scratch;
mod sessions;
mod update;
mod visibility;
mod window;

use std::error::Error;
use std::path::PathBuf;

use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::WindowId;

use trame::log_error;
use trame::session::SessionEvent;
use trame::workspace::InstanceId;

use pending::PendingOffer;
use persist::{Role, Saved};
use power::Process;
use running::{FolderTarget, Offer, Restart, Running};
use update::UpdateStatus;

#[derive(Clone, Debug)]
pub enum UserEvent {
    Session(WindowId, SessionEvent),
    Update(UpdateStatus),
    Folder(WindowId, FolderTarget, PathBuf),
    Icon(WindowId, InstanceId, u64),
    Open,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    if let Some(path) = persist::log_path() {
        trame::files::start_log(path);
    }
    let mut role = persist::claim();
    if !role.owns() && persist::app_dir().is_some_and(|folder| door::knock(&folder)) {
        return Ok(());
    }
    let saved = persist::load(&mut role);
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    if let Some(folder) = role.folder() {
        let visitor = proxy.clone();
        door::listen(folder, move || {
            let _ = visitor.send_event(UserEvent::Open);
        });
    }
    let notifier = proxy.clone();
    let update = update::start(move |status| {
        let _ = notifier.send_event(UserEvent::Update(status));
    });
    let mut app = App {
        proxy,
        update,
        offer: PendingOffer::default(),
        role,
        saved,
        windows: Vec::new(),
        process: Process::default(),
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
    saved: Vec<Saved>,
    windows: Vec<Running>,
    process: Process,
    failure: Option<Box<dyn Error>>,
}

impl App {
    fn updated(&mut self, event_loop: &ActiveEventLoop, status: UpdateStatus) {
        let status = match status {
            UpdateStatus::Installed(handoff) => {
                match self.restart(event_loop, || handoff.start()) {
                    Ok(()) => return,
                    Err(error) => {
                        log_error!("the new version did not start: {error}");
                        UpdateStatus::Failed
                    }
                }
            }
            UpdateStatus::AvailableAtStart(version) if self.updates_on_start() => {
                self.offer.send(Offer { version }, &mut self.windows);
                return self.restart_for(event_loop, Restart::Update);
            }
            UpdateStatus::Available(version) | UpdateStatus::AvailableAtStart(version) => {
                return self.offer.send(Offer { version }, &mut self.windows);
            }
            status => status,
        };
        for running in &mut self.windows {
            running.set_update(&status);
        }
        self.update = status;
    }

    fn updates_on_start(&self) -> bool {
        self.windows
            .first()
            .is_some_and(|running| running.preferences().update_on_start)
    }
}
