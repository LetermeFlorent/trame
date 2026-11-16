#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod fatal;
#[cfg(gnome_ui)]
mod gnome;

use std::error::Error;

fn main() {
    if let Err(error) = start() {
        fatal::report(&format!("trame could not start: {error}"));
        std::process::exit(1);
    }
}

#[cfg(gnome_ui)]
fn start() -> Result<(), Box<dyn Error>> {
    if gnome::wanted() {
        gnome::run()
    } else {
        app::run()
    }
}

#[cfg(not(gnome_ui))]
fn start() -> Result<(), Box<dyn Error>> {
    app::run()
}
