use trame::files::IconStore;
use trame::geometry::Rect;
use trame::render::Scene;
use trame::text::TextSystem;
use trame::theme::Theme;
use trame::ui::chrome::ChromeInput;
use trame::ui::{FrameLayout, FrameView, Hit, Motion};

pub fn paint_with(
    text: &mut TextSystem,
    frame: &FrameLayout,
    theme: &Theme,
    hover: Option<Hit>,
    draw: impl FnOnce(&mut Scene, &mut TextSystem, &ChromeInput),
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

#[track_caller]
pub fn bubble_beside(scene: &Scene, theme: &Theme, rail: Rect, middle: f32) -> f32 {
    let bubble = scene
        .quads()
        .iter()
        .find(|quad| quad.color == theme.chrome.to_array())
        .expect("a bubble is drawn");
    let [x, y, width, height] = bubble.rect;
    assert!(x >= rail.right());
    assert!(y <= middle && y + height >= middle);
    width
}
