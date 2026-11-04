mod brand;
mod fill;
mod outline;
mod tabbar;
mod titlebar;

use trame::geometry::Rect;
use trame::render::Scene;
use trame::theme::Rgba;

fn painted_at(scene: &Scene, rect: Rect, color: Rgba) -> bool {
    scene
        .quads()
        .iter()
        .any(|quad| quad.rect[..2] == [rect.x, rect.y] && quad.color == color.to_array())
}
