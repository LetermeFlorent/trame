use std::sync::Arc;

use async_channel::{Receiver, Sender};
use gtk::glib;
use trame::session::{Notifier, SessionEvent};

pub fn channel() -> (Notifier, Receiver<SessionEvent>) {
    let (sender, receiver) = async_channel::unbounded();
    let notify: Notifier = Arc::new(move |event| {
        let _ = sender.send_blocking(event);
    });
    (notify, receiver)
}

pub fn listen(
    receiver: Receiver<SessionEvent>,
    mut handle: impl FnMut(SessionEvent) -> bool + 'static,
) {
    glib::spawn_future_local(async move {
        while let Ok(event) = receiver.recv().await {
            if !handle(event) {
                break;
            }
        }
    });
}

pub fn unbounded_listen<T: 'static>(mut handle: impl FnMut(T) -> bool + 'static) -> Sender<T> {
    let (sender, receiver) = async_channel::unbounded();
    glib::spawn_future_local(async move {
        while let Ok(item) = receiver.recv().await {
            if !handle(item) {
                break;
            }
        }
    });
    sender
}
