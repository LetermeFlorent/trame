use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;
use trame::geometry::Point;
use trame::theme::Appearance;
use trame::ui::chrome::{ChromeInput, LoaderView};

use super::WindowCtx;
use super::timer::{refresh, settle_now};
use crate::gnome::canvas::{build_chrome, build_overlays, render_layer};

pub fn wire_canvases(ctx: &Rc<WindowCtx>) {
    let cs = Rc::downgrade(ctx);
    ctx.chrome.set_draw(Box::new(move |snap, w, h, sc| {
        if let Some(c) = cs.upgrade() {
            draw_chrome_layer(&c, snap, (w, h, sc));
        }
    }));
    let os = Rc::downgrade(ctx);
    ctx.overlay.set_draw(Box::new(move |snap, w, h, sc| {
        if let Some(c) = os.upgrade() {
            draw_overlay_layer(&c, snap, (w, h, sc));
        }
    }));
    let pk = Rc::downgrade(ctx);
    ctx.overlay.set_pick(Box::new(move |x, y| {
        pk.upgrade().is_some_and(|c| {
            let s = c.state.borrow();
            let sc = f64::from(s.window_scale.max(1.0));
            s.overlay_picks(Point::new((x * sc) as f32, (y * sc) as f32))
        })
    }));
    let ps = Rc::downgrade(ctx);
    ctx.picture.on_state(move |open| {
        if let Some(c) = ps.upgrade() {
            c.state.borrow_mut().picture_open = open;
            refresh(&c);
        }
    });
}

fn draw_chrome_layer(ctx: &WindowCtx, snap: &gtk::Snapshot, size: (i32, i32, i32)) {
    let mut s = ctx.state.borrow_mut();
    let s = &mut *s;
    let input = ChromeInput {
        theme: &s.desk.theme,
        frame: &s.frame,
        view: &s.view,
        motion: &s.motion,
        icons: &s.desk.icons,
    };
    build_chrome(&mut s.chrome_scene, &mut s.text, &input);
    render_layer(
        snap,
        size,
        &mut ctx.chrome_painter.borrow_mut(),
        &s.chrome_scene,
        &mut s.text,
    );
}

fn draw_overlay_layer(ctx: &WindowCtx, snap: &gtk::Snapshot, size: (i32, i32, i32)) {
    let mut s = ctx.state.borrow_mut();
    let drag = s.drag_view();
    let loader: Option<LoaderView> = None;
    let s = &mut *s;
    let input = ChromeInput {
        theme: &s.desk.theme,
        frame: &s.frame,
        view: &s.view,
        motion: &s.motion,
        icons: &s.desk.icons,
    };
    build_overlays(
        &mut s.overlay_scene,
        &mut s.text,
        &input,
        drag.as_ref(),
        loader.as_ref(),
    );
    render_layer(
        snap,
        size,
        &mut ctx.overlay_painter.borrow_mut(),
        &s.overlay_scene,
        &mut s.text,
    );
}

pub fn window_signals(ctx: &Rc<WindowCtx>) {
    let c1 = Rc::downgrade(ctx);
    ctx.window.connect_close_request(move |_| {
        if let Some(c) = c1.upgrade() {
            c.state.borrow_mut().ask_close(trame::desk::Closing::Window);
            refresh(&c);
            settle_now(&c);
        }
        glib::Propagation::Stop
    });
    let c2 = Rc::downgrade(ctx);
    ctx.window.connect_is_active_notify(move |win| {
        if let Some(c) = c2.upgrade() {
            let focused = win.is_active();
            c.state.borrow_mut().view.window_focused = focused;
            if focused {
                c.notices.borrow_mut().clear(Some(&c.app));
            }
            refresh(&c);
        }
    });
    theme_and_maximize_signals(ctx);
}

fn theme_and_maximize_signals(ctx: &Rc<WindowCtx>) {
    let c3 = Rc::downgrade(ctx);
    adw::StyleManager::default().connect_dark_notify(move |mgr| {
        if let Some(c) = c3.upgrade()
            && mgr.system_supports_color_schemes()
        {
            let app = if mgr.is_dark() {
                Appearance::Dark
            } else {
                Appearance::Light
            };
            c.state.borrow_mut().set_appearance(app);
            refresh(&c);
        }
    });
    let c4 = Rc::downgrade(ctx);
    ctx.window.connect_maximized_notify(move |win| {
        if let Some(c) = c4.upgrade() {
            c.state.borrow_mut().maximized = win.is_maximized();
            c.state.borrow_mut().changed();
            refresh(&c);
        }
    });
}
