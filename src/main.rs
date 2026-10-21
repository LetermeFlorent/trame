#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

fn main() {
    if let Err(error) = app::run() {
        eprintln!("trame: {error}");
        std::process::exit(1);
    }
}
