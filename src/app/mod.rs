mod graphics;
mod handler;
mod notice;
mod persist;
mod power;
mod running;
mod sessions;
mod update;
mod visibility;
mod window;

use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};

use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};

use trame::preferences::Updates;
use trame::session::SessionEvent;
use trame::workspace::{InstanceId, Snapshot};

use running::{FolderTarget, Running};
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
    let saved = persist::load();
    let updates = saved
        .as_ref()
        .map_or_else(Updates::default, |snapshot| snapshot.preferences.updates);
    let update = update::start(updates, move |status| {
        let _ = notifier.send_event(UserEvent::Update(status));
    });
    let mut app = App {
        proxy,
        update,
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
    saved: Option<Snapshot>,
    running: Option<Running>,
    failure: Option<Box<dyn Error>>,
}

impl App {
    fn launch(&mut self, event_loop: &ActiveEventLoop) -> Result<Running, Box<dyn Error>> {
        let saved = self.saved.take();
        Running::open_window(event_loop, self.proxy.clone(), saved)?.show(&self.update)
    }

    fn relaunch(&mut self, exe: &Path) -> io::Result<()> {
        if let Some(running) = &mut self.running {
            running.save_for_restart();
        }
        update::relaunch(exe)
    }

    fn updated(&mut self, event_loop: &ActiveEventLoop, status: UpdateStatus) {
        let status = match status {
            UpdateStatus::Installed(exe) => match self.relaunch(&exe) {
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
                if let Some(running) = &mut self.running {
                    running.offer_update(version);
                }
                return;
            }
            status => status,
        };
        if let Some(running) = &mut self.running {
            running.set_update(&status);
        }
        self.update = status;
    }
}
