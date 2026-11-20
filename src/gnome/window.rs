use adw::prelude::*;
use gtk::glib;
use trame::session::pty::default_shell;

use super::picture::PictureOverlay;
use super::view::TermView;

const APP_TITLE: &str = "trame";
const DEFAULT_WIDTH: i32 = 960;
const DEFAULT_HEIGHT: i32 = 620;

pub fn open(app: &adw::Application) {
    let view = TermView::new();
    let picture = PictureOverlay::new(&view);
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&view));
    overlay.add_overlay(picture.widget());
    wire_picture(&view, &picture);
    let title = adw::WindowTitle::new(APP_TITLE, &shell_name());
    let header = adw::HeaderBar::builder().title_widget(&title).build();
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&overlay));
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .default_width(DEFAULT_WIDTH)
        .default_height(DEFAULT_HEIGHT)
        .content(&toolbar)
        .build();
    attach_overlay_keys(&window, &picture);
    wire_close(&view, &window);
    window.present();
    view.grab_focus();
}

fn wire_picture(view: &TermView, picture: &PictureOverlay) {
    let worker = view.worker();
    let opener = picture.clone();
    view.on_picture(move |source, path| {
        opener.open(source, path, &worker);
    });
    let decoder = picture.clone();
    view.on_overlay_ready(move |job, image| {
        decoder.finish_decode(job, image);
    });
    let styler = picture.clone();
    view.on_theme(move |theme| {
        styler.apply_theme(theme);
    });
}

fn attach_overlay_keys(window: &adw::ApplicationWindow, picture: &PictureOverlay) {
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let target = picture.clone();
    keys.connect_key_pressed(move |_, keyval, _, _| {
        glib::Propagation::from(target.handle_key(keyval))
    });
    window.add_controller(keys);
}

fn wire_close(view: &TermView, window: &adw::ApplicationWindow) {
    let closing = window.downgrade();
    view.on_exit(move || {
        if let Some(window) = closing.upgrade() {
            window.close();
        }
    });
}

fn shell_name() -> String {
    let shell = default_shell();
    shell
        .program
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
