mod events;
mod view;
mod window;

use std::env;
use std::error::Error;

use adw::prelude::*;
use gtk::{gio, glib};

const APP_ID: &str = "io.github.LetermeFlorent.Trame";
const UI_VARIABLE: &str = "TRAME_UI";
const CLASSIC_UI: &str = "classic";

pub fn wanted() -> bool {
    env::var(UI_VARIABLE).map_or(true, |value| value != CLASSIC_UI)
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(window::open);
    if app.run_with_args::<&str>(&[]) == glib::ExitCode::SUCCESS {
        Ok(())
    } else {
        Err("the GTK application stopped with an error".into())
    }
}
