use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::{App, UserEvent};

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
            UserEvent::Icon(id, image) => {
                if let Some(running) = &mut self.running {
                    running.icon_picked(id, image);
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
