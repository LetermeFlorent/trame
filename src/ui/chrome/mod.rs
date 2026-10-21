mod icons;
mod sidebar;
mod tabbar;
mod titlebar;

pub use icons::{Icon, Pen, draw_icon};

use crate::color::Rgba;
use crate::geometry::Rect;
use crate::render::Scene;
use crate::text::{TextLayout, TextSystem};
use crate::theme::Theme;
use crate::ui::frame::FrameLayout;
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;
use crate::ui::paint::FrameView;
use crate::workspace::Axis;

const ICON_STROKE: f32 = 1.25;
const SMALL_RADIUS: f32 = 3.0;
const ACTIVE_DIVIDER: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Press {
    Idle,
    Hover,
    Pressed,
}

struct Painter<'a> {
    scene: &'a mut Scene,
    text: &'a mut TextSystem,
    theme: &'a Theme,
    metrics: UiMetrics,
    view: &'a FrameView,
}

impl Painter<'_> {
    fn state(&self, hit: Hit) -> Press {
        match (self.view.hover == Some(hit), self.view.pressed == Some(hit)) {
            (true, true) => Press::Pressed,
            (true, false) => Press::Hover,
            _ => Press::Idle,
        }
    }

    fn button(&mut self, rect: Rect, hit: Hit, icon: Icon) {
        let radius = self.metrics.px(UiMetrics::RADIUS);
        self.icon_button(rect, hit, icon, radius);
    }

    fn small_button(&mut self, rect: Rect, hit: Hit, icon: Icon) {
        let radius = self.metrics.px(SMALL_RADIUS);
        self.icon_button(rect, hit, icon, radius);
    }

    fn icon_button(&mut self, rect: Rect, hit: Hit, icon: Icon, radius: f32) {
        let state = self.state(hit);
        let background = match state {
            Press::Idle => None,
            Press::Hover => Some(self.theme.hover),
            Press::Pressed => Some(self.theme.pressed),
        };
        if let Some(color) = background {
            self.scene.fill(rect, color, radius);
        }
        let color = match state {
            Press::Idle => self.theme.text_muted,
            _ => self.theme.text,
        };
        let size = self.metrics.px(UiMetrics::ICON);
        self.icon(icon, centered(rect, size), color);
    }

    fn icon(&mut self, icon: Icon, rect: Rect, color: Rgba) {
        let stroke = (ICON_STROKE * self.metrics.scale).round().max(1.0);
        let mut pen = Pen {
            scene: self.scene,
            color,
            stroke,
        };
        draw_icon(&mut pen, icon, rect);
    }

    fn label(&mut self, layout: &TextLayout, x: f32, rect: Rect, color: Rgba) {
        let baseline = (rect.y + (rect.height + layout.ascent - layout.descent) / 2.0).round();
        self.text
            .draw_layout(self.scene, layout, x.round(), baseline, color);
    }

    fn rule(&mut self, rect: Rect, color: Rgba) {
        self.scene.fill(rect, color, 0.0);
    }
}

fn centered(rect: Rect, size: f32) -> Rect {
    Rect::new(
        (rect.x + (rect.width - size) / 2.0).round(),
        (rect.y + (rect.height - size) / 2.0).round(),
        size,
        size,
    )
}

pub fn draw_chrome(
    scene: &mut Scene,
    text: &mut TextSystem,
    theme: &Theme,
    frame: &FrameLayout,
    view: &FrameView,
) {
    let mut painter = Painter {
        scene,
        text,
        theme,
        metrics: frame.metrics,
        view,
    };
    titlebar::draw(&mut painter, &frame.title_bar, frame.sidebar.is_some());
    if let Some(sidebar) = &frame.sidebar {
        sidebar::draw(&mut painter, sidebar);
    }
    for pane in &frame.panes {
        tabbar::draw(&mut painter, pane);
    }
}

pub fn draw_dividers(scene: &mut Scene, theme: &Theme, frame: &FrameLayout, view: &FrameView) {
    let active = |index: usize| {
        let hit = Some(Hit::Divider(index));
        view.pressed == hit || (view.pressed.is_none() && view.hover == hit)
    };
    let grow = ((frame.metrics.px(ACTIVE_DIVIDER) - frame.metrics.line()) / 2.0).max(0.0);
    for (index, divider) in frame.dividers.iter().enumerate() {
        if !active(index) {
            scene.fill(divider.rect, theme.border, 0.0);
            continue;
        }
        let rect = match divider.axis {
            Axis::Row => divider.rect.inset_by(-grow, 0.0),
            Axis::Column => divider.rect.inset_by(0.0, -grow),
        };
        scene.fill(rect, theme.accent, 0.0);
    }
}
