mod icons;
pub mod logo;
mod overlay;
mod painter;
mod sidebar;
mod tabbar;
mod titlebar;

pub use icons::{Icon, Pen, draw_icon};
pub use overlay::{
    LoaderProgress, LoaderView, draw_editor, draw_loader, draw_menu, draw_settings, draw_tooltip,
};

use self::painter::{Painter, Press, centered};
use crate::render::Scene;
use crate::text::TextSystem;
use crate::theme::Theme;
use crate::ui::frame::FrameLayout;
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;
use crate::ui::motion::Motion;
use crate::ui::paint::FrameView;

const ACTIVE_DIVIDER: f32 = 2.0;
const SIDEBAR_SHOWN: f32 = 0.5;

pub struct ChromeInput<'a> {
    pub theme: &'a Theme,
    pub frame: &'a FrameLayout,
    pub view: &'a FrameView,
    pub motion: &'a Motion,
}

pub fn draw_chrome(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let frame = input.frame;
    let mut painter = Painter::new(scene, text, input);
    let shown = frame.reveal >= SIDEBAR_SHOWN;
    titlebar::draw(&mut painter, &frame.title_bar, shown);
    sidebar::draw_panel(&mut painter, frame.side);
    if let Some(rail) = &frame.rail {
        painter.scene.push_opacity(1.0 - frame.reveal);
        sidebar::draw_rail(&mut painter, rail);
        painter.scene.pop_opacity();
    }
    if let Some(sidebar) = &frame.sidebar {
        painter.scene.push_clip(frame.side);
        painter.scene.push_opacity(frame.reveal);
        sidebar::draw(&mut painter, sidebar);
        painter.scene.pop_opacity();
        painter.scene.pop_clip();
    }
    let open = frame.settings.is_some();
    sidebar::draw_footer(&mut painter, &frame.footer, open, frame.reveal);
    for pane in &frame.panes {
        tabbar::draw(&mut painter, pane);
    }
}

pub fn draw_dividers(scene: &mut Scene, theme: &Theme, frame: &FrameLayout, view: &FrameView) {
    let active =
        |hit: Hit| view.pressed == Some(hit) || (view.pressed.is_none() && view.hover == Some(hit));
    let grow = ((frame.metrics.px(ACTIVE_DIVIDER) - frame.metrics.line()) / 2.0).max(0.0);
    if active(Hit::SidebarEdge) {
        let grab = frame.metrics.px(UiMetrics::DIVIDER_GRAB);
        scene.fill(
            frame.sidebar_edge().inset_by(grab - grow, 0.0),
            theme.accent,
            0.0,
        );
    }
    for (index, divider) in frame.dividers.iter().enumerate() {
        if !active(Hit::Divider(index)) {
            scene.fill(divider.rect, theme.border, 0.0);
            continue;
        }
        let rect = divider.widened(grow);
        scene.fill(rect, theme.accent, 0.0);
    }
}
