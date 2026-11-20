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
    tint: Option<trame::theme::Swatch>,
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

    pub fn sync_modals(&self, state: &WindowState, picture: &PictureOverlay) {
        let blocked = picture.is_open() || state.modal_open();
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
        let live: Vec<SessionId> = state_rc
            .borrow()
            .desk
            .sessions
            .map()
            .keys()
            .copied()
            .collect();
        self.prune(&live);
        let panes = state_rc.borrow().frame.panes.clone();
        let mut shown = Vec::new();
        for pane in panes.iter().filter(|p| p.offline.is_none()) {
            let Some(id) = pane.active.filter(|id| live.contains(id)) else {
                continue;
            };
            let slot = PaneSlot {
                id,
                rect: pane.terminal,
                scale: scale.max(1.0),
                tint: pane.tint,
            };
            self.place_one(slot, state_rc, picture, configure);
            shown.push(id);
        }
        for (&id, view) in &self.views {
            if !shown.contains(&id) {
                view.set_visible(false);
            }
        }
        if let Some(cell) = self.views.values().next().map(TermView::cell_size) {
            state_rc.borrow_mut().cell_size = cell;
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
        let state = state_rc.borrow();
        view.set_theme(state.desk.theme.tinted(slot.tint));
        view.set_font_size(state.desk.preferences.font_size);
        view.set_detect_links(state.desk.preferences.detect_links);
        view.set_cwd(state.desk.sessions.cwd(slot.id));
        view.set_scroll(state.motion.scroll_offset(slot.id));
        drop(state);
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
