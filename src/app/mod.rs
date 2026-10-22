mod graphics;
mod persist;
mod running;
mod saver;
mod sessions;
mod update;
mod visibility;
mod window;

use std::error::Error;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::WindowId;

use trame::session::SessionEvent;

use running::Running;
use update::Update;

#[derive(Clone, Copy, Debug)]
pub enum UserEvent {
    Session(SessionEvent),
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let update = Update::start();
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let mut app = App {
        proxy: event_loop.create_proxy(),
        update: Some(update),
        running: None,
        failure: None,
    };
    event_loop.run_app(&mut app)?;
    app.failure.map_or(Ok(()), Err)
}

struct App {
    proxy: EventLoopProxy<UserEvent>,
    update: Option<Update>,
    running: Option<Running>,
    failure: Option<Box<dyn Error>>,
}

impl App {
    fn launch(&mut self, event_loop: &ActiveEventLoop) -> Result<Option<Running>, Box<dyn Error>> {
        let opened = Running::open_window(event_loop, self.proxy.clone())?;
        if self.update.take().is_some_and(Update::relaunched) {
            return Ok(None);
        }
        opened.show().map(Some)
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_some() {
            return;
        }
        match self.launch(event_loop) {
            Ok(Some(running)) => self.running = Some(running),
            Ok(None) => event_loop.exit(),
            Err(error) => {
                self.failure = Some(error);
                event_loop.exit();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        if let Some(running) = &mut self.running {
            running.user_event(event_loop, event);
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
