use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use super::running::Running;
use super::{App, UserEvent};

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            self.open_saved(event_loop);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Update(status) => self.updated(event_loop, status),
            UserEvent::Open => self.open_another(event_loop),
            UserEvent::Session(id, event) => {
                self.on(event_loop, id, |running| {
                    running.session_event(event_loop, event)
                });
            }
            UserEvent::Folder(id, target, folder) => {
                self.on(event_loop, id, |running| {
                    running.folder_picked(target, folder)
                });
            }
            UserEvent::Icon(id, tab, image) => {
                self.on(event_loop, id, |running| running.icon_picked(tab, image));
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.on(event_loop, id, |running| {
            running.window_event(event_loop, event)
        });
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let wake = self
            .windows
            .iter_mut()
            .filter_map(Running::about_to_wait)
            .min();
        self.settle(event_loop);
        event_loop.set_control_flow(wake.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.windows.iter_mut().for_each(Running::save_on_exit);
        self.windows.clear();
    }
}
