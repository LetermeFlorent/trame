use adw::prelude::*;
use trame::session::pty::default_shell;

use super::view::TermView;

const APP_TITLE: &str = "trame";
const DEFAULT_WIDTH: i32 = 960;
const DEFAULT_HEIGHT: i32 = 620;

pub fn open(app: &adw::Application) {
    let view = TermView::new();
    let title = adw::WindowTitle::new(APP_TITLE, &shell_name());
    let header = adw::HeaderBar::builder().title_widget(&title).build();
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&view));
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .default_width(DEFAULT_WIDTH)
        .default_height(DEFAULT_HEIGHT)
        .content(&toolbar)
        .build();
    let closing = window.downgrade();
    view.on_exit(move || {
        if let Some(window) = closing.upgrade() {
            window.close();
        }
    });
    window.present();
    view.grab_focus();
}

fn shell_name() -> String {
    let shell = default_shell();
    shell
        .program
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
