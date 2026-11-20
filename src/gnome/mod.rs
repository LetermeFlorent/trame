mod canvas;
mod events;
mod fleet;
mod input;
mod notify;
mod panes;
mod picture;
mod state;
mod view;
mod window;
mod worker;

use std::cell::RefCell;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

use adw::prelude::*;
use gtk::{gio, glib};
use trame::host::door;
use trame::host::persist;
use trame::host::update::{self, UpdateStatus};

use fleet::Fleet;

const APP_ID: &str = "io.github.LetermeFlorent.Trame";
const UI_VARIABLE: &str = "TRAME_UI";
const CLASSIC_UI: &str = "classic";

pub fn wanted() -> bool {
    env::var(UI_VARIABLE).map_or(true, |value| value != CLASSIC_UI)
}

pub fn run() -> Result<(), Box<dyn Error>> {
    if let Some(path) = persist::log_path() {
        trame::files::start_log(path);
    }
    let mut role = persist::claim();
    let opened = launch_folder(env::args_os());
    let knocked = |folder: PathBuf| door::knock(&folder, opened.as_deref());
    if !role.owns() && persist::app_dir().is_some_and(knocked) {
        return Ok(());
    }
    let saved = persist::load(&mut role);
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    let weak_slot: Rc<RefCell<Weak<RefCell<Fleet>>>> = Rc::new(RefCell::new(Weak::new()));
    let fleet_cell: Rc<RefCell<Fleet>> = Rc::new(RefCell::new(Fleet::new(
        role,
        saved,
        opened,
        UpdateStatus::Done,
        builder(&app, &weak_slot),
    )));
    let builder_weak = Rc::downgrade(&fleet_cell);
    *weak_slot.borrow_mut() = builder_weak.clone();
    wire_background(&app, &fleet_cell, &builder_weak);
    let hold = Rc::clone(&fleet_cell);
    app.connect_activate(move |a| {
        if hold.borrow().windows.is_empty() {
            hold.borrow_mut().open_saved(a);
        }
    });
    if app.run_with_args::<&str>(&[]) == glib::ExitCode::SUCCESS {
        Ok(())
    } else {
        Err("the GTK application stopped with an error".into())
    }
}

fn builder(
    app: &adw::Application,
    fleet_slot: &Rc<RefCell<Weak<RefCell<Fleet>>>>,
) -> fleet::BuilderFn {
    let app = app.clone();
    let slot = Rc::clone(fleet_slot);
    Box::new(move |key, win_slot, shared, spec, update| {
        window::build_handle(window::BuildRequest {
            app: &app,
            fleet: &slot.borrow(),
            key,
            slot: win_slot,
            shared,
            spec,
            update,
        })
    })
}

fn wire_background(
    app: &adw::Application,
    fleet: &Rc<RefCell<Fleet>>,
    weak: &Weak<RefCell<Fleet>>,
) {
    let weak_ref = weak.clone();
    if let Some(folder) = fleet.borrow().role.folder().map(PathBuf::from) {
        let open_tx = events::unbounded_listen(move |path| {
            if let Some(f) = weak_ref.upgrade() {
                f.borrow_mut().open_another(path);
                return true;
            }
            false
        });
        door::listen(&folder, move |opened| {
            let _ = open_tx.send_blocking(opened);
        });
    }
    let update_app = app.clone();
    let update_fleet = Rc::downgrade(fleet);
    let update_tx = events::unbounded_listen(move |status| {
        if let Some(f) = update_fleet.upgrade() {
            f.borrow_mut().updated(&update_app, status);
            return true;
        }
        false
    });
    let initial = update::start(move |status| {
        let _ = update_tx.send_blocking(status);
    });
    fleet.borrow_mut().update = initial;
}

fn launch_folder(arguments: impl Iterator<Item = OsString>) -> Option<PathBuf> {
    let arguments: Vec<PathBuf> = arguments.skip(1).map(PathBuf::from).collect();
    let flagged = arguments
        .iter()
        .any(|argument| argument.to_string_lossy().starts_with("--"));
    let folder = arguments.into_iter().find(|argument| argument.is_dir());
    folder.filter(|_| !flagged)
}
