use adw::prelude::*;
use trame::host::key::WindowKey;
use trame::host::quit::Quit;
use trame::log_error;

use super::{Fleet, OpenSpec, WindowHandle};
use crate::gnome::state::drag::{Drag, Lifted};

impl Fleet {
    pub fn settle(&mut self, app: &adw::Application, source: Option<WindowKey>) {
        if let Some(key) = source {
            self.share_preferences(key);
        }
        self.handle_tear_offs();
        let quits: Vec<(WindowKey, Quit)> = self
            .windows
            .iter()
            .filter_map(|w| w.state.borrow_mut().desk.take_quit().map(|q| (w.key, q)))
            .collect();
        for (key, quit) in quits {
            match quit {
                Quit::Close => self.close_one(app, key),
                Quit::Restart(r) => self.restart_for(app, r),
            }
        }
        let rests = self
            .windows
            .iter()
            .all(|w| w.state.borrow().desk.power.rests());
        let hidden = self
            .windows
            .iter()
            .all(|w| w.state.borrow().desk.power.hidden());
        self.process.follow(rests, hidden);
    }

    fn share_preferences(&mut self, key: WindowKey) {
        let Some(prefs) = self
            .windows
            .iter()
            .find(|w| w.key == key)
            .map(|w| w.state.borrow().desk.preferences.clone())
        else {
            return;
        };
        if self
            .windows
            .iter()
            .all(|w| w.state.borrow().desk.preferences == prefs)
        {
            return;
        }
        for w in &self.windows {
            let mut s = w.state.borrow_mut();
            let effects = s.desk.follow_preferences(&prefs);
            s.apply(effects);
            drop(s);
            (w.refresh)();
        }
    }

    fn handle_tear_offs(&mut self) {
        let Some(spec) = self.windows.iter().find_map(take_lifted) else {
            return;
        };
        let slot = self.free_window_slot();
        let _ = self.spawn_window(slot, spec);
    }

    fn close_one(&mut self, app: &adw::Application, key: WindowKey) {
        if self.windows.len() <= 1 {
            return self.close_all(app);
        }
        let Some(idx) = self.windows.iter().position(|w| w.key == key) else {
            return;
        };
        let handle = self.windows.remove(idx);
        handle.state.borrow_mut().desk.transcripts.flush();
        let slot_idx = handle.state.borrow().desk.saver.slot().index();
        if let Err(err) = self.role.slot(slot_idx).forget() {
            log_error!("a closed window's files could not be removed: {err}");
        }
        handle.window.destroy();
    }

    fn close_all(&mut self, app: &adw::Application) {
        self.exiting = true;
        for w in &self.windows {
            let mut s = w.state.borrow_mut();
            let snap = s.snapshot();
            let effects = s.desk.save_on_exit(snap);
            s.apply(effects);
            s.desk.transcripts.flush();
        }
        app.quit();
    }
}

fn take_lifted(w: &WindowHandle) -> Option<OpenSpec> {
    let mut s = w.state.borrow_mut();
    let out = match s.drag {
        Some(Drag::Out(l)) => l,
        _ => return None,
    };
    s.drag = None;
    let spec = match out {
        Lifted::Tab(tab) => {
            let torn = s.desk.tear_tab(tab);
            s.apply(torn.effects);
            OpenSpec::Torn(torn.value?)
        }
        Lifted::Instance(id) => {
            let moved = s.desk.give_instance(id);
            s.apply(moved.effects);
            OpenSpec::Moved(moved.value?)
        }
    };
    drop(s);
    (w.refresh)();
    Some(spec)
}
