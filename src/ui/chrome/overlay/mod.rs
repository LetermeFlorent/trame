mod connect;
mod editor;
mod loader;
mod menu;
mod popup;
mod settings;
mod tooltip;

pub use editor::draw_editor;
pub use loader::{LoaderProgress, LoaderView, draw_loader};
pub use menu::draw_menu;
pub use popup::draw_confirm;
pub use settings::draw_settings;
pub use tooltip::draw_tooltip;

use crate::theme::Rgba;

const ON_ACCENT: Rgba = Rgba::hex(0xFFFFFF);
