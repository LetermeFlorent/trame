use trame::render::Scene;
use trame::text::TextSystem;
use trame::ui::chrome::{self, ChromeInput, LoaderView};
use trame::ui::drop::{DragView, draw_drag};

const ATLAS_ATTEMPTS: usize = 2;

pub fn build_chrome(scene: &mut Scene, text: &mut TextSystem, input: &ChromeInput) {
    for _ in 0..ATLAS_ATTEMPTS {
        scene.reset(input.frame.bounds.width, input.frame.bounds.height);
        chrome::draw_chrome(scene, text, input);
        for pane in &input.frame.panes {
            let tinted = input.theme.tinted(pane.tint);
            if pane.radius > 0.0 {
                let line = input.frame.metrics.line();
                scene.border(pane.card, tinted.border_strong, pane.radius, line);
            } else {
                scene.fill(pane.body, tinted.terminal, 0.0);
            }
        }
        chrome::draw_dividers(scene, input.theme, input.frame, input.view);
        if !text.end_frame() {
            break;
        }
    }
}

pub fn build_overlays(
    scene: &mut Scene,
    text: &mut TextSystem,
    input: &ChromeInput,
    drag: Option<&DragView>,
    loader: Option<&LoaderView>,
) {
    for _ in 0..ATLAS_ATTEMPTS {
        scene.reset(input.frame.bounds.width, input.frame.bounds.height);
        chrome::draw_offline(scene, text, input);
        chrome::draw_editor(scene, text, input);
        chrome::draw_settings(scene, text, input);
        chrome::draw_menu(scene, text, input);
        chrome::draw_message(scene, text, input);
        match drag {
            Some(drag) => draw_drag(scene, text, input, drag),
            None if input.frame.menu.is_none() => chrome::draw_tooltip(scene, text, input),
            None => {}
        }
        if let Some(loader) = loader {
            chrome::draw_loader(scene, text, input, loader);
        }
        if !text.end_frame() {
            break;
        }
    }
}
