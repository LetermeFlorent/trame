use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, glib};
use trame::ui::ResizeEdge;

use super::timer::{refresh, settle_now};
use super::{WindowCtx, picker, pointer};
use crate::gnome::input::clipboard::{copy_active, paste_into_terminal};
use crate::gnome::input::keys::DecodedKey;
use crate::gnome::input::{CaptureOutcome, handle_capture_key, update_modifiers};
use crate::gnome::state::{WindowAction, WindowState};

pub fn attach(ctx: &Rc<WindowCtx>) {
    attach_keys(ctx);
    pointer::attach_pointer(ctx);
    pointer::attach_scroll(ctx);
    pointer::attach_drop(ctx);
}

fn attach_keys(ctx: &Rc<WindowCtx>) {
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let pressed = Rc::downgrade(ctx);
    keys.connect_key_pressed(move |_, keyval, keycode, mask| {
        let Some(c) = pressed.upgrade() else {
            return glib::Propagation::Proceed;
        };
        let Some(decoded) = DecodedKey::from_gdk(keyval, keycode, mask) else {
            return glib::Propagation::Proceed;
        };
        let exiting = c.fleet.upgrade().is_some_and(|f| f.borrow().exiting);
        let outcome = handle_capture_key(&mut c.state.borrow_mut(), exiting, &decoded);
        handle_outcome(&c, outcome)
    });
    let released = Rc::downgrade(ctx);
    keys.connect_key_released(move |_, _, _, mask| {
        if let Some(c) = released.upgrade() {
            update_modifiers(&mut c.state.borrow_mut(), mask);
            refresh(&c);
        }
    });
    ctx.window.add_controller(keys);
}

fn handle_outcome(ctx: &Rc<WindowCtx>, outcome: CaptureOutcome) -> glib::Propagation {
    let display = gtk::prelude::WidgetExt::display(&ctx.window);
    match outcome {
        CaptureOutcome::Pass => glib::Propagation::Proceed,
        CaptureOutcome::Consumed => {
            refresh(ctx);
            settle_now(ctx);
            glib::Propagation::Stop
        }
        CaptureOutcome::ClosePicture => {
            ctx.picture.close();
            glib::Propagation::Stop
        }
        CaptureOutcome::Copy => {
            copy_active(&mut ctx.state.borrow_mut(), Some(&display));
            refresh(ctx);
            glib::Propagation::Stop
        }
        CaptureOutcome::Paste => {
            let ref_c = Rc::clone(ctx);
            paste_into_terminal(&ctx.state, Some(&display), move || {
                refresh(&ref_c);
                settle_now(&ref_c);
            });
            glib::Propagation::Stop
        }
    }
}

pub fn run_pending_action(ctx: &Rc<WindowCtx>) {
    let action = ctx.state.borrow_mut().pending_action.take();
    if let Some(act) = action {
        apply_action(ctx, act);
    }
    let picker_job = ctx.state.borrow_mut().picker_req.take();
    if let Some(job) = picker_job {
        picker::launch(ctx, job);
    }
}

fn apply_action(ctx: &WindowCtx, action: WindowAction) {
    match action {
        WindowAction::Minimize => ctx.window.minimize(),
        WindowAction::ToggleMaximize if ctx.window.is_maximized() => ctx.window.unmaximize(),
        WindowAction::ToggleMaximize => ctx.window.maximize(),
        WindowAction::Raise => ctx.window.present(),
        WindowAction::Move => begin_surface_drag(&ctx.window, &ctx.state, None),
        WindowAction::Resize(edge) => begin_surface_drag(&ctx.window, &ctx.state, Some(edge)),
    }
}

fn begin_surface_drag(
    window: &adw::ApplicationWindow,
    state: &std::cell::RefCell<WindowState>,
    edge: Option<ResizeEdge>,
) {
    let Some(top) = window
        .surface()
        .and_then(|s| s.downcast::<gdk::Toplevel>().ok())
    else {
        return;
    };
    let Some(dev) = gtk::prelude::WidgetExt::display(window)
        .default_seat()
        .and_then(|s| s.pointer())
    else {
        return;
    };
    let (x, y) = {
        let s = state.borrow();
        let sc = f64::from(s.window_scale.max(1.0));
        let pt = s.pointer.unwrap_or_default();
        (f64::from(pt.x) / sc, f64::from(pt.y) / sc)
    };
    let btn = gdk::BUTTON_PRIMARY as i32;
    match edge {
        None => top.begin_move(&dev, btn, x, y, gdk::CURRENT_TIME),
        Some(e) => top.begin_resize(gdk_edge(e), Some(&dev), btn, x, y, gdk::CURRENT_TIME),
    }
}

fn gdk_edge(edge: ResizeEdge) -> gdk::SurfaceEdge {
    match edge {
        ResizeEdge::North => gdk::SurfaceEdge::North,
        ResizeEdge::South => gdk::SurfaceEdge::South,
        ResizeEdge::West => gdk::SurfaceEdge::West,
        ResizeEdge::East => gdk::SurfaceEdge::East,
        ResizeEdge::NorthWest => gdk::SurfaceEdge::NorthWest,
        ResizeEdge::NorthEast => gdk::SurfaceEdge::NorthEast,
        ResizeEdge::SouthWest => gdk::SurfaceEdge::SouthWest,
        ResizeEdge::SouthEast => gdk::SurfaceEdge::SouthEast,
    }
}
