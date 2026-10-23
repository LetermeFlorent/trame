mod graphics;
mod persist;
mod running;
mod sessions;
mod update;
mod visibility;
mod window;

use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::WindowId;

use trame::session::SessionEvent;

use running::{FolderTarget, Running};
use update::UpdateStatus;

#[derive(Clone, Debug)]
pub enum UserEvent {
    Session(SessionEvent),
    Update(UpdateStatus),
    Folder(FolderTarget, PathBuf),
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let notifier = proxy.clone();
    let update = update::start(move |status| {
        let _ = notifier.send_event(UserEvent::Update(status));
    });
    let mut app = App {
        proxy,
        update,
        running: None,
        failure: None,
    };
    event_loop.run_app(&mut app)?;
    app.failure.map_or(Ok(()), Err)
}

struct App {
    proxy: EventLoopProxy<UserEvent>,
    update: UpdateStatus,
    running: Option<Running>,
    failure: Option<Box<dyn Error>>,
}

impl App {
    fn launch(&mut self, event_loop: &ActiveEventLoop) -> Result<Running, Box<dyn Error>> {
        Running::open_window(event_loop, self.proxy.clone())?.show(&self.update)
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

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_some() {
            return;
        }
        match self.launch(event_loop) {
            Ok(running) => self.running = Some(running),
            Err(error) => {
                self.failure = Some(error);
                event_loop.exit();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Update(status) => self.updated(event_loop, status),
            UserEvent::Session(event) => {
                if let Some(running) = &mut self.running {
                    running.session_event(event_loop, event);
                }
            }
            UserEvent::Folder(target, folder) => {
                if let Some(running) = &mut self.running {
                    running.folder_picked(target, folder);
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Some(running) = &mut self.running {
            running.window_event(event_loop, event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(running) = &mut self.running {
            running.about_to_wait(event_loop);
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        if let Some(running) = &mut self.running {
            running.save_on_exit();
        }
        self.running = None;
    }
}
