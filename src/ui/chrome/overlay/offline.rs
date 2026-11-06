use crate::render::Scene;
use crate::text::TextSystem;
use crate::ui::chrome::{ChromeInput, Painter};
use crate::ui::hit::Hit;

use super::popup::{LabelButton, draw_label_button};

const SHADE: f32 = 0.75;

pub fn draw_offline(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    let mut painter = Painter::new(scene, text, input);
    for pane in &input.frame.panes {
        let Some(card) = &pane.offline else {
            continue;
        };
        let theme = input.theme.tinted(pane.tint);
        let shade = theme.terminal.with_alpha(SHADE);
        painter.scene.fill(pane.body, shade, pane.radius);
        let x = card.line.centered_x(card.message.width);
        painter.label(&card.message, x, card.line, theme.text);
        let button = LabelButton {
            rect: card.button,
            label: &card.label,
            hit: Hit::Disconnect(card.tab),
            filled: false,
            faint: false,
        };
        draw_label_button(&mut painter, &button);
    }
}
