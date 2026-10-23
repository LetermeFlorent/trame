use crate::render::Scene;
use crate::text::TextSystem;
use crate::ui::chrome::{ChromeInput, Painter};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

const MENU_BLUR: f32 = 8.0;

pub fn draw_menu(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let Some(menu) = &input.frame.menu else {
        return;
    };
    let mut painter = Painter::new(scene, text, input);
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    painter.panel(menu.rect, MENU_BLUR);
    for row in &menu.rows {
        painter.press_fill(row.rect, Hit::MenuEntry(row.entry), radius);
        painter.label(&row.label, row.label_x, row.rect, input.theme.text);
    }
}
