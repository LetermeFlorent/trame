use super::ChromeInput;
use super::icons::{Icon, Pen, draw_icon};
use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::text::{TextLayout, TextSystem};
use crate::theme::{Rgba, Theme};
use crate::ui::frame::SettingLine;
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;
use crate::ui::motion::Motion;
use crate::ui::paint::FrameView;

const ICON_STROKE: f32 = 1.25;
const SMALL_RADIUS: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Press {
    Idle,
    Hover,
    Pressed,
}

pub(super) struct Painter<'a> {
    pub(super) scene: &'a mut Scene,
    pub(super) text: &'a mut TextSystem,
    pub(super) theme: &'a Theme,
    pub(super) metrics: UiMetrics,
    pub(super) view: &'a FrameView,
    pub(super) motion: &'a Motion,
}

impl<'a> Painter<'a> {
    pub(super) fn new(
        scene: &'a mut Scene,
        text: &'a mut TextSystem,
        input: &ChromeInput<'a>,
    ) -> Self {
        Self {
            scene,
            text,
            theme: input.theme,
            metrics: input.frame.metrics,
            view: input.view,
            motion: input.motion,
        }
    }

    pub(super) fn panel(&mut self, rect: Rect, blur: f32) {
        let theme = self.theme;
        let radius = self.metrics.px(UiMetrics::RADIUS);
        let line = self.metrics.line();
        let blur = self.metrics.px(blur);
        self.scene.shadow(rect, theme.shadow, radius, blur);
        self.scene.fill(rect, theme.chrome, radius);
        self.scene.border(rect, theme.border_strong, radius, line);
    }

    pub(super) fn state(&self, hit: Hit) -> Press {
        match (self.view.hover == Some(hit), self.view.pressed == Some(hit)) {
            (true, true) => Press::Pressed,
            (true, false) => Press::Hover,
            _ => Press::Idle,
        }
    }

    pub(super) fn button(&mut self, rect: Rect, hit: Hit, icon: Icon) {
        let radius = self.metrics.px(UiMetrics::RADIUS);
        self.icon_button(rect, hit, icon, radius);
    }

    pub(super) fn small_button(&mut self, rect: Rect, hit: Hit, icon: Icon) {
        let radius = self.metrics.px(SMALL_RADIUS);
        self.icon_button(rect, hit, icon, radius);
    }

    pub(super) fn glow(&self, hit: Hit) -> f32 {
        self.motion.glow(hit)
    }

    fn icon_button(&mut self, rect: Rect, hit: Hit, icon: Icon, radius: f32) {
        let glow = self.press_fill(rect, hit, radius);
        let size = self.metrics.px(UiMetrics::ICON);
        let color = self.theme.muted_text(glow);
        self.icon(icon, centered(rect, size), color);
    }

    pub(super) fn press_fill(&mut self, rect: Rect, hit: Hit, radius: f32) -> f32 {
        let theme = self.theme;
        let (fill, glow) = match self.state(hit) {
            Press::Pressed => (theme.pressed, 1.0),
            _ => {
                let glow = self.glow(hit);
                (theme.hover.faded(glow), glow)
            }
        };
        self.scene.fill(rect, fill, radius);
        glow
    }

    pub(super) fn icon(&mut self, icon: Icon, rect: Rect, color: Rgba) {
        let stroke = (ICON_STROKE * self.metrics.scale).round().max(1.0);
        let mut pen = Pen {
            scene: self.scene,
            color,
            stroke,
        };
        draw_icon(&mut pen, icon, rect);
    }

    pub(super) fn label(&mut self, layout: &TextLayout, x: f32, rect: Rect, color: Rgba) {
        let pen = Point::new(x.round(), layout.baseline_in(rect));
        self.text.draw_layout(self.scene, layout, pen, color);
    }

    pub(super) fn setting_line(&mut self, line: &SettingLine, color: Rgba) {
        self.label(&line.label, line.label_x, line.rect, color);
    }

    pub(super) fn rule(&mut self, rect: Rect, color: Rgba) {
        self.scene.fill(rect, color, 0.0);
    }
}

pub(super) fn centered(rect: Rect, size: f32) -> Rect {
    Rect::new(
        rect.centered_x(size).round(),
        rect.centered_y(size).round(),
        size,
        size,
    )
}
