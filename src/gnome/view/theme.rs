use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, graphene};
use trame::theme::{Appearance, Rgba, Theme};

use super::TermView;

pub fn follow(view: &TermView) {
    let manager = adw::StyleManager::default();
    apply(view, manager.is_dark());
    let watched = view.downgrade();
    manager.connect_dark_notify(move |manager| {
        if let Some(view) = watched.upgrade() {
            apply(&view, manager.is_dark());
        }
    });
}

fn apply(view: &TermView, dark: bool) {
    let appearance = if dark {
        Appearance::Dark
    } else {
        Appearance::Light
    };
    let theme = Theme::new(appearance);
    let imp = view.imp();
    if let Some(session) = imp.session.borrow().as_ref() {
        let colors = theme.reported_colors();
        session.terminal().screen_mut().set_reported_colors(colors);
    }
    if let Some(changed) = imp.theme_changed.borrow().as_ref() {
        changed(&theme);
    }
    imp.theme.replace(theme);
    view.queue_draw();
}

pub fn rgba(color: Rgba) -> gdk::RGBA {
    gdk::RGBA::new(color.r, color.g, color.b, color.a)
}

pub fn fill(snapshot: &gtk::Snapshot, color: Rgba, area: graphene::Rect) {
    snapshot.append_color(&rgba(color), &area);
}
