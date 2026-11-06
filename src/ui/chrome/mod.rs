mod icons;
pub mod logo;
mod overlay;
mod painter;
mod sidebar;
mod tabbar;
mod titlebar;

pub use icons::Icon;
pub use overlay::{
    LoaderProgress, LoaderView, draw_editor, draw_loader, draw_menu, draw_message, draw_offline,
    draw_settings, draw_tooltip,
};

use self::painter::{Painter, Press, centered};
use crate::files::IconStore;
use crate::render::Scene;
use crate::text::TextSystem;
use crate::theme::Theme;
use crate::ui::frame::FrameLayout;
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;
use crate::ui::motion::Motion;
use crate::ui::paint::FrameView;

const ACTIVE_DIVIDER: f32 = 2.0;

pub struct ChromeInput<'a> {
    pub theme: &'a Theme,
    pub frame: &'a FrameLayout,
    pub view: &'a FrameView,
    pub motion: &'a Motion,
    pub icons: &'a IconStore,
}

pub fn draw_chrome(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let frame = input.frame;
    let mut painter = Painter::new(scene, text, input);
    let shown = frame.side.width > 0.0;
    titlebar::draw(&mut painter, &frame.title_bar, shown);
    if shown {
        painter.scene.push_clip(frame.side);
        draw_side(&mut painter, frame);
        painter.scene.pop_clip();
    }
    for pane in &frame.panes {
        tabbar::draw(&mut painter, pane);
    }
}

fn draw_side(painter: &mut Painter, frame: &FrameLayout) {
    sidebar::draw_panel(painter, frame.side);
    if let Some(rail) = &frame.rail {
        painter.scene.push_opacity(1.0 - frame.reveal);
        sidebar::draw_rail(painter, rail);
        painter.scene.pop_opacity();
    }
    if let Some(sidebar) = &frame.sidebar {
        painter.scene.push_opacity(frame.reveal);
        sidebar::draw(painter, sidebar);
        painter.scene.pop_opacity();
    }
    let open = frame.settings.is_some();
    sidebar::draw_footer(painter, &frame.footer, open, frame.reveal);
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
