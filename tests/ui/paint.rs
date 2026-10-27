use trame::files::IconStore;
use trame::render::Scene;
use trame::text::TextSystem;
use trame::theme::Theme;
use trame::ui::chrome::ChromeInput;
use trame::ui::{FrameLayout, FrameView, Hit, Motion};

pub type Draw = fn(&mut Scene, &mut TextSystem, &ChromeInput);

pub fn paint_with(
    text: &mut TextSystem,
    frame: &FrameLayout,
    theme: &Theme,
    hover: Option<Hit>,
    draw: Draw,
) -> Scene {
    let view = FrameView {
        hover,
        pressed: None,
        window_focused: true,
    };
    let motion = Motion::default();
    let input = ChromeInput {
        theme,
        frame,
        view: &view,
        motion: &motion,
        icons: &IconStore::default(),
    };
    let mut scene = Scene::new(frame.bounds.width, frame.bounds.height);
    draw(&mut scene, text, &input);
    scene
}
