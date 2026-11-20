use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gtk::prelude::ObjectExt;
use trame::session::SessionId;
use trame::term::Terminal;

use super::WindowCtx;
use super::timer::refresh;
use crate::gnome::input::clipboard::{paste_primary, sync_primary};
use crate::gnome::picture::PictureOverlay;
use crate::gnome::state::WindowState;
use crate::gnome::view::{BoundSession, TermView};

pub fn configure_pane(
    id: SessionId,
    view: &TermView,
    state_rc: &std::cell::RefCell<WindowState>,
    picture: &PictureOverlay,
    ctx: &Rc<WindowCtx>,
) {
    let (terminal, cwd, label) = {
        let s = state_rc.borrow();
        let Some(session) = s.desk.sessions.get(id) else {
            return;
        };
        (
            session.shared_terminal(),
            s.desk.sessions.cwd(id),
            s.desk.strings.open_in_viewer,
        )
    };
    view.bind(build_bound(id, ctx, terminal, cwd));
    wire_pane_callbacks(view, picture, ctx, label);
}

fn build_bound(
    id: SessionId,
    ctx: &Rc<WindowCtx>,
    terminal: Arc<Mutex<Terminal>>,
    cwd: Option<PathBuf>,
) -> BoundSession {
    let c_write = Rc::downgrade(ctx);
    let c_focus = Rc::downgrade(ctx);
    BoundSession {
        id,
        terminal,
        cwd,
        writer: Box::new(move |bytes| {
            if let Some(c) = c_write.upgrade()
                && let Some(sess) = c.state.borrow().desk.sessions.get(id)
            {
                sess.terminal().screen_mut().reset_display();
                sess.write(bytes);
            }
        }),
        focus: Box::new(move || focus_pane_session(&c_focus, id)),
    }
}

fn focus_pane_session(weak: &std::rc::Weak<WindowCtx>, id: SessionId) {
    let Some(c) = weak.upgrade() else {
        return;
    };
    let mut s = c.state.borrow_mut();
    let Some(pane) = s
        .frame
        .panes
        .iter()
        .find(|p| p.active == Some(id))
        .map(|p| p.id)
    else {
        return;
    };
    if s.desk.workspace.active_tab() == Some(id) {
        return;
    }
    s.desk.focus_pane(pane);
    s.changed();
    drop(s);
    refresh(&c);
}

fn wire_pane_callbacks(
    view: &TermView,
    picture: &PictureOverlay,
    ctx: &Rc<WindowCtx>,
    open_label: &'static str,
) {
    let pic = picture.clone();
    let view_ref = view.downgrade();
    let c_pic = Rc::downgrade(ctx);
    view.on_picture(move |source, path| {
        let Some(v) = view_ref.upgrade() else { return };
        let label = c_pic
            .upgrade()
            .map_or(open_label, |c| c.state.borrow().desk.strings.open_in_viewer);
        pic.open_from(&v, source, path, label);
    });
    let dec = picture.clone();
    view.on_overlay_ready(move |job, img| dec.finish_decode(job, img));
    let c_sel = Rc::downgrade(ctx);
    view.on_primary_selection(move |text| {
        if let Some(c) = c_sel.upgrade() {
            sync_primary(&text, Some(&gtk::prelude::WidgetExt::display(&c.window)));
        }
    });
    let c_mid = Rc::downgrade(ctx);
    view.on_middle_click(move || {
        if let Some(c) = c_mid.upgrade() {
            let ref_c = Rc::clone(&c);
            paste_primary(
                &c.state,
                Some(&gtk::prelude::WidgetExt::display(&c.window)),
                move || {
                    refresh(&ref_c);
                },
            );
        }
    });
}
