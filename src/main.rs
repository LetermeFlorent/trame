#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod fatal;

fn main() {
    if let Err(error) = app::run() {
        fatal::report(&format!("trame could not start: {error}"));
        std::process::exit(1);
    }
}
