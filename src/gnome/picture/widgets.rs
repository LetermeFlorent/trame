use gtk::graphene;
use gtk::prelude::*;
use trame::files::pictures::open_image_file;
use trame::locale::system_language;

use super::PictureOverlay;
use super::style::{CLOSE_PAD, CLOSE_SIZE, OPEN_GAP, PICTURE_MARGIN};

const SCRIM_CLASS: &str = "trame-picture-scrim";
const BUTTON_CLASS: &str = "trame-picture-btn";
const OPEN_CLASS: &str = "trame-picture-open";
const POINTER_CURSOR: &str = "pointer";

pub fn build_scrim() -> (gtk::Overlay, gtk::Picture) {
    let root = gtk::Overlay::new();
    root.add_css_class(SCRIM_CLASS);
    root.set_visible(false);
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.set_can_target(true);
    let picture = gtk::Picture::new();
    picture.set_can_shrink(true);
    picture.set_content_fit(gtk::ContentFit::Contain);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture.set_margin_top(PICTURE_MARGIN);
    picture.set_margin_bottom(PICTURE_MARGIN);
    picture.set_margin_start(PICTURE_MARGIN);
    picture.set_margin_end(PICTURE_MARGIN);
    root.set_child(Some(&picture));
    (root, picture)
}

pub fn build_buttons(close_glyph: &gtk::DrawingArea) -> (gtk::Box, gtk::Button, gtk::Button) {
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, OPEN_GAP);
    actions.set_halign(gtk::Align::End);
    actions.set_valign(gtk::Align::Start);
    actions.set_margin_top(CLOSE_PAD);
    actions.set_margin_end(CLOSE_PAD);
    let open_button = gtk::Button::with_label(system_language().strings().open_in_viewer);
    open_button.add_css_class(BUTTON_CLASS);
    open_button.add_css_class(OPEN_CLASS);
    open_button.set_cursor_from_name(Some(POINTER_CURSOR));
    open_button.set_visible(false);
    let close_button = gtk::Button::new();
    close_button.add_css_class(BUTTON_CLASS);
    close_button.set_size_request(CLOSE_SIZE, CLOSE_SIZE);
    close_button.set_cursor_from_name(Some(POINTER_CURSOR));
    close_button.set_child(Some(close_glyph));
    actions.append(&open_button);
    actions.append(&close_button);
    (actions, open_button, close_button)
}

pub fn attach_interactions(overlay: &PictureOverlay, close_button: &gtk::Button) {
    let opener = overlay.clone();
    overlay.open_widget().connect_clicked(move |_| {
        if let Some(path) = opener.current_path() {
            open_image_file(path);
        }
    });
    let closer = overlay.clone();
    close_button.connect_clicked(move |_| closer.close());
    let scrim = overlay.clone();
    let click = gtk::GestureClick::new();
    click.set_button(0);
    click.connect_pressed(move |gesture, _, x, y| {
        let on_open = point_on_open(&scrim, x as f32, y as f32);
        gesture.set_state(gtk::EventSequenceState::Claimed);
        if gesture.current_button() == gtk::gdk::BUTTON_PRIMARY && !on_open {
            scrim.close();
        }
    });
    overlay.widget().add_controller(click);
    let flags =
        gtk::EventControllerScrollFlags::BOTH_AXES | gtk::EventControllerScrollFlags::DISCRETE;
    let scroll = gtk::EventControllerScroll::new(flags);
    scroll.connect_scroll(|_, _, _| gtk::glib::Propagation::Stop);
    overlay.widget().add_controller(scroll);
}

fn point_on_open(overlay: &PictureOverlay, x: f32, y: f32) -> bool {
    let point = graphene::Point::new(x, y);
    let button = overlay.open_widget();
    button.is_visible()
        && button
            .compute_bounds(overlay.widget())
            .is_some_and(|bounds| bounds.contains_point(&point))
}
