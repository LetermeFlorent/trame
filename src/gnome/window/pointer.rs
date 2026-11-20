use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, gio, glib};
use trame::geometry::Point;
use trame::ui::Hit;

use super::WindowCtx;
use super::timer::{refresh, settle_now};
use crate::gnome::input::drop_scroll::apply_file_drop;
use crate::gnome::input::scroll::scroll_lines;
use crate::gnome::input::update_modifiers;

pub(super) fn attach_pointer(ctx: &Rc<WindowCtx>) {
    let stack = ctx.window.content().expect("window stack");
    let motion = gtk::EventControllerMotion::new();
    motion.set_propagation_phase(gtk::PropagationPhase::Capture);
    let moved = Rc::downgrade(ctx);
    motion.connect_motion(move |ctrl, x, y| {
        if let Some(c) = moved.upgrade() {
            let mut s = c.state.borrow_mut();
            let sc = s.window_scale.max(1.0);
            update_modifiers(&mut s, ctrl.current_event_state());
            s.pointer_moved(Point::new(x as f32 * sc, y as f32 * sc));
            drop(s);
            refresh(&c);
            settle_now(&c);
        }
    });
    let left = Rc::downgrade(ctx);
    motion.connect_leave(move |_| {
        if let Some(c) = left.upgrade() {
            c.state.borrow_mut().pointer_left();
            refresh(&c);
        }
    });
    stack.add_controller(motion);
    attach_clicks(ctx, &stack);
}

fn attach_clicks(ctx: &Rc<WindowCtx>, stack: &gtk::Widget) {
    let click = gtk::GestureClick::new();
    click.set_button(0);
    click.set_propagation_phase(gtk::PropagationPhase::Capture);
    let pressed = Rc::downgrade(ctx);
    click.connect_pressed(move |g, _, x, y| {
        if let Some(c) = pressed.upgrade()
            && claims_chrome(&c, x, y)
        {
            g.set_state(gtk::EventSequenceState::Claimed);
            handle_click(&c, true, g.current_button(), (x, y));
        }
    });
    let released = Rc::downgrade(ctx);
    click.connect_released(move |g, _, x, y| {
        if let Some(c) = released.upgrade()
            && (claims_chrome(&c, x, y)
                || c.state.borrow().drag.is_some()
                || c.state.borrow().pressed_hit.is_some())
        {
            g.set_state(gtk::EventSequenceState::Claimed);
            handle_click(&c, false, g.current_button(), (x, y));
        }
    });
    stack.add_controller(click);
}

fn claims_chrome(ctx: &WindowCtx, x: f64, y: f64) -> bool {
    let s = ctx.state.borrow();
    if s.picture_open {
        return false;
    }
    let sc = s.window_scale.max(1.0);
    let pt = Point::new(x as f32 * sc, y as f32 * sc);
    if s.overlay_picks(pt) {
        return true;
    }
    let resizable = !s.maximized && !s.fullscreen;
    !matches!(s.frame.hit(pt, resizable), Some(Hit::Terminal(_)) | None)
}

fn handle_click(ctx: &Rc<WindowCtx>, pressed: bool, button: u32, at: (f64, f64)) {
    let exiting = ctx.fleet.upgrade().is_some_and(|f| f.borrow().exiting);
    let mut s = ctx.state.borrow_mut();
    let sc = s.window_scale.max(1.0);
    s.pointer_moved(Point::new(at.0 as f32 * sc, at.1 as f32 * sc));
    s.mouse_button(exiting, pressed, button);
    drop(s);
    refresh(ctx);
    settle_now(ctx);
}

pub(super) fn attach_scroll(ctx: &Rc<WindowCtx>) {
    let stack = ctx.window.content().expect("window stack");
    let flags = gtk::EventControllerScrollFlags::BOTH_AXES;
    let scroll = gtk::EventControllerScroll::new(flags);
    scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
    let target = Rc::downgrade(ctx);
    scroll.connect_scroll(move |ctrl, _, dy| {
        let Some(c) = target.upgrade() else {
            return glib::Propagation::Proceed;
        };
        if c.state.borrow().picture_open {
            return glib::Propagation::Proceed;
        }
        let pixel = ctrl.unit() == gdk::ScrollUnit::Surface;
        scroll_lines(&mut c.state.borrow_mut(), dy as f32, pixel);
        refresh(&c);
        glib::Propagation::Stop
    });
    stack.add_controller(scroll);
}

pub(super) fn attach_drop(ctx: &Rc<WindowCtx>) {
    let drop_target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    let weak = Rc::downgrade(ctx);
    drop_target.connect_drop(move |_, val, x, y| {
        let Some(c) = weak.upgrade() else {
            return false;
        };
        let Ok(list) = val.get::<gdk::FileList>() else {
            return false;
        };
        let files: Vec<_> = list
            .files()
            .into_iter()
            .filter_map(|f: gio::File| f.path())
            .collect();
        let refs: Vec<&Path> = files.iter().map(Path::new).collect();
        let sc = c.state.borrow().window_scale.max(1.0);
        let pt = Some(Point::new(x as f32 * sc, y as f32 * sc));
        let ok = apply_file_drop(&mut c.state.borrow_mut(), pt, &refs);
        if ok {
            refresh(&c);
        }
        ok
    });
    ctx.window.add_controller(drop_target);
}
