use std::rc::Rc;
use std::time::{Duration, Instant};

use adw::prelude::*;
use gtk::glib;
use trame::session::SessionEvent;

use super::WindowCtx;
use super::actions::configure_pane;
use super::input::run_pending_action;

const MIN_WAKE_WAIT: Duration = Duration::from_millis(4);

pub fn refresh(ctx: &Rc<WindowCtx>) {
    sync_geometry(ctx);
    run_pending_action(ctx);
    sync_panes_and_chrome(ctx);
    schedule_wake(ctx);
}

pub fn settle_now(ctx: &Rc<WindowCtx>) {
    if let Some(fleet) = ctx.fleet.upgrade() {
        fleet.borrow_mut().settle(&ctx.app, Some(ctx.key));
    }
}

pub fn on_session_event(ctx: &Rc<WindowCtx>, event: SessionEvent) {
    let exiting = ctx.fleet.upgrade().is_some_and(|f| f.borrow().exiting);
    let output_id = ctx.state.borrow_mut().session_event(exiting, event);
    if let Some(id) = output_id {
        let s = ctx.state.borrow();
        let display = gtk::prelude::WidgetExt::display(&ctx.window);
        ctx.notices
            .borrow_mut()
            .forward(&s, id, Some(&ctx.app), Some(&display));
    }
    refresh(ctx);
    settle_now(ctx);
}

fn sync_geometry(ctx: &WindowCtx) {
    let scale = ctx.window.scale_factor().max(1) as f32;
    let w = (ctx.chrome.width().max(ctx.window.width()).max(1)) as f32 * scale;
    let h = (ctx.chrome.height().max(ctx.window.height()).max(1)) as f32 * scale;
    let mut s = ctx.state.borrow_mut();
    if (s.window_scale - scale).abs() > 0.01 {
        s.window_scale = scale;
        s.window_size = (w, h);
        s.apply_scale();
    } else if (s.window_size.0 - w).abs() > 0.5 || (s.window_size.1 - h).abs() > 0.5 {
        s.window_size = (w, h);
        s.relayout();
        s.dirty = true;
    }
}

fn sync_panes_and_chrome(ctx: &Rc<WindowCtx>) {
    let scale = ctx.state.borrow().window_scale.max(1.0);
    let mut panes = ctx.panes.borrow_mut();
    panes.sync_layout(
        &ctx.state,
        &ctx.picture,
        scale,
        &mut |id, view, state_rc, pic| {
            configure_pane(id, view, state_rc, pic, ctx);
        },
    );
    let s = ctx.state.borrow();
    panes.sync_modals(&s, &ctx.picture);
    ctx.picture.apply_theme(&s.desk.theme);
    ctx.window.set_cursor_from_name(Some(s.cursor));
    ctx.chrome.queue_draw();
    ctx.overlay.queue_draw();
}

fn schedule_wake(ctx: &Rc<WindowCtx>) {
    if let Some(old) = ctx.timer.borrow_mut().take() {
        old.remove();
    }
    let now = Instant::now();
    let next = ctx.state.borrow_mut().about_to_wait(now);
    let dirty = ctx.state.borrow().dirty;
    if dirty {
        ctx.state.borrow_mut().dirty = false;
        ctx.chrome.queue_draw();
        ctx.overlay.queue_draw();
    }
    let Some(at) = next else { return };
    let wait = at.saturating_duration_since(now).max(MIN_WAKE_WAIT);
    let weak = Rc::downgrade(ctx);
    let id = glib::timeout_add_local_once(wait, move || {
        if let Some(c) = weak.upgrade() {
            *c.timer.borrow_mut() = None;
            refresh(&c);
            settle_now(&c);
        }
    });
    *ctx.timer.borrow_mut() = Some(id);
}
