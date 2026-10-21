mod blocks;
mod lines;
mod shapes;
mod table;

use crate::color::Rgba;
use crate::geometry::Rect;
use crate::render::Scene;

use table::Special;

pub fn is_drawn(ch: char) -> bool {
    matches!(u32::from(ch), 0x2500..=0x259F)
}

pub fn draw(scene: &mut Scene, ch: char, rect: Rect, color: Rgba, light: f32) -> bool {
    if blocks::draw_block(scene, ch, rect, color) {
        return true;
    }
    let Some(code) = table::line_code(ch) else {
        return false;
    };
    match table::special(ch) {
        Some(Special::Dashed(count)) => shapes::draw_dashes(scene, code, count, rect, color, light),
        Some(Special::Arc) => shapes::draw_arc(scene, code, rect, color, light),
        Some(Special::Diagonal { rising, falling }) => {
            shapes::draw_diagonals(scene, rect, color, light, rising, falling);
        }
        None => lines::draw_lines(scene, code, rect, color, light),
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell() -> Rect {
        Rect::new(0.0, 0.0, 8.0, 17.0)
    }

    fn quads(ch: char) -> usize {
        let mut scene = Scene::new(8.0, 17.0);
        assert!(draw(&mut scene, ch, cell(), Rgba::hex(0xffffff), 1.0));
        scene.quads().len()
    }

    #[test]
    fn every_box_and_block_character_is_drawn() {
        for code in 0x2500..=0x259F {
            let ch = char::from_u32(code).unwrap();
            assert!(is_drawn(ch));
            assert!(quads(ch) > 0, "{ch} draws nothing");
        }
    }

    #[test]
    fn other_characters_are_left_to_the_font() {
        let mut scene = Scene::new(8.0, 17.0);
        assert!(!draw(&mut scene, 'a', cell(), Rgba::hex(0xffffff), 1.0));
        assert!(!is_drawn('a'));
        assert!(scene.quads().is_empty());
    }

    #[test]
    fn straight_lines_span_the_whole_cell() {
        let mut scene = Scene::new(8.0, 17.0);
        draw(&mut scene, '─', cell(), Rgba::hex(0xffffff), 1.0);
        let left = scene
            .quads()
            .iter()
            .map(|quad| quad.rect[0])
            .fold(f32::MAX, f32::min);
        let right = scene
            .quads()
            .iter()
            .map(|quad| quad.rect[0] + quad.rect[2])
            .fold(0.0, f32::max);
        assert_eq!((left, right), (0.0, 8.0));
    }

    #[test]
    fn double_lines_draw_two_strokes_per_arm() {
        assert_eq!(quads('═'), 4);
        assert_eq!(quads('╬'), 8);
        assert_eq!(quads('┼'), 4);
    }
}
