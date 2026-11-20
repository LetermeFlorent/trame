use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use trame::geometry::Rect;
use trame::session::SessionId;

use super::picture::PictureOverlay;
use super::state::WindowState;
use super::view::TermView;

pub type ConfigureViewFn<'a> =
    &'a mut dyn FnMut(SessionId, &TermView, &Rc<RefCell<WindowState>>, &PictureOverlay);

struct PaneSlot {
    id: SessionId,
    rect: Rect,
    scale: f32,
}

pub struct PaneViews {
    fixed: gtk::Fixed,
    views: HashMap<SessionId, TermView>,
}

impl PaneViews {
    pub fn new() -> Self {
        let fixed = gtk::Fixed::new();
        fixed.set_hexpand(true);
        fixed.set_vexpand(true);
        fixed.set_overflow(gtk::Overflow::Hidden);
        Self {
            fixed,
            views: HashMap::new(),
        }
    }

    pub fn widget(&self) -> &gtk::Fixed {
        &self.fixed
    }

    pub fn views(&self) -> &HashMap<SessionId, TermView> {
        &self.views
    }

    pub fn active_view(&self, state: &WindowState) -> Option<&TermView> {
        let id = state.desk.workspace.active_tab()?;
        self.views.get(&id)
    }

    pub fn sync_modals(&self, state: &WindowState, picture: &PictureOverlay) {
        let blocked = picture.is_open()
            || state.frame.blocked
            || state.desk.closing.is_some()
            || state.menu.is_some()
            || state.editing.is_some()
            || state.settings.is_some();
        for view in self.views.values() {
            view.set_picture_open(blocked);
        }
    }

    pub fn sync_layout(
        &mut self,
        state_rc: &Rc<RefCell<WindowState>>,
        picture: &PictureOverlay,
        scale: f32,
        configure: ConfigureViewFn,
    ) {
        let live_ids: Vec<SessionId> = state_rc
            .borrow()
            .desk
            .sessions
            .map()
            .keys()
            .copied()
            .collect();
        self.prune(&live_ids);
        let panes = state_rc.borrow().frame.panes.clone();
        let mut shown = Vec::new();
        for pane in &panes {
            let Some(id) = pane.active.filter(|id| pane.offline.is_none() && live_ids.contains(id))
            else {
                continue;
            };
            let slot = PaneSlot {
                id,
                rect: pane.terminal,
                scale: scale.max(1.0),
            };
            self.place_one(slot, state_rc, picture, configure);
            shown.push(id);
        }
        for (&id, view) in &self.views {
            if !shown.contains(&id) {
                view.set_visible(false);
            }
        }
    }

    fn prune(&mut self, live_ids: &[SessionId]) {
        self.views.retain(|id, view| {
            let keep = live_ids.contains(id);
            if !keep {
                self.fixed.remove(view);
            }
            keep
        });
    }

    fn place_one(
        &mut self,
        slot: PaneSlot,
        state_rc: &Rc<RefCell<WindowState>>,
        picture: &PictureOverlay,
        configure: ConfigureViewFn,
    ) {
        let is_new = !self.views.contains_key(&slot.id);
        let view = self.views.entry(slot.id).or_default();
        if is_new {
            self.fixed.put(view, 0.0, 0.0);
            configure(slot.id, view, state_rc, picture);
        }
        let x = f64::from(slot.rect.x / slot.scale).round();
        let y = f64::from(slot.rect.y / slot.scale).round();
        let w = (slot.rect.width / slot.scale).round().max(1.0) as i32;
        let h = (slot.rect.height / slot.scale).round().max(1.0) as i32;
        view.set_size_request(w, h);
        self.fixed.move_(view, x, y);
        view.set_visible(true);
        view.queue_draw();
    }
}

impl Default for PaneViews {
    fn default() -> Self {
        Self::new()
    }
}
