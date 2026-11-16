use std::sync::{Arc, Mutex, PoisonError};

use winit::event_loop::EventLoopProxy;
use winit::window::WindowId;

use trame::session::Notifier;

use super::UserEvent;

pub fn session_notifier(proxy: EventLoopProxy<UserEvent>, window: WindowId) -> Notifier {
    let proxy = Mutex::new(proxy);
    Arc::new(move |event| {
        let proxy = proxy.lock().unwrap_or_else(PoisonError::into_inner);
        let _ = proxy.send_event(UserEvent::Session(window, event));
    })
}
