use trame::render::Scene;
use trame::theme::{Appearance, Theme};
use trame::ui::chrome::LoaderView;

use super::{paint, waiting};
use crate::{frame, text_system, workspace};

fn painted(loader: &LoaderView) -> (Scene, f32) {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &workspace());
    let scene = paint(&mut text, &layout, &theme, loader);
    (scene, layout.metrics.px(1.0))
}

#[test]
fn a_shifted_loader_moves_its_content_but_not_the_backdrop() {
    let (still, _) = painted(&waiting(1.0));
    let (moved, unit) = painted(&LoaderView {
        shift: 8.0,
        ..waiting(1.0)
    });
    assert_eq!(still.quads().len(), moved.quads().len());
    for (index, (before, after)) in still.quads().iter().zip(moved.quads()).enumerate() {
        let drop = if index < 2 { 0.0 } else { 8.0 * unit };
        assert!((after.rect[1] - before.rect[1] - drop).abs() <= 1.0);
        assert!((after.rect[0] - before.rect[0]).abs() <= 1.0);
    }
}

#[test]
fn hidden_content_leaves_only_the_backdrop() {
    let (scene, _) = painted(&LoaderView {
        content: 0.0,
        ..waiting(1.0)
    });
    assert_eq!(scene.quads().len(), 2);
}

#[test]
fn a_changing_message_shows_both_texts_until_the_blend_ends() {
    let settled = painted(&waiting(1.0)).0.quads().len();
    let blending = |blend| LoaderView {
        previous: Some("Wait"),
        blend,
        ..waiting(1.0)
    };
    assert!(painted(&blending(0.5)).0.quads().len() > settled);
    assert_eq!(painted(&blending(1.0)).0.quads().len(), settled);
    assert!(painted(&blending(0.0)).0.quads().len() < settled);
}
