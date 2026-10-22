mod graphics;
mod persist;
mod running;
mod sessions;
mod visibility;
mod window;

use std::error::Error;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::WindowId;

use trame::session::SessionEvent;

use running::Running;

#[derive(Clone, Copy, Debug)]
pub enum UserEvent {
    Session(SessionEvent),
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let mut app = App {
        proxy: event_loop.create_proxy(),
        running: None,
        failure: None,
    };
    event_loop.run_app(&mut app)?;
    app.failure.map_or(Ok(()), Err)
}

struct App {
    proxy: EventLoopProxy<UserEvent>,
    running: Option<Running>,
    failure: Option<Box<dyn Error>>,
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_some() {
            return;
        }
        match Running::start(event_loop, self.proxy.clone()) {
            Ok(running) => self.running = Some(running),
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
            running.save_session();
        }
        self.running = None;
    }
}
