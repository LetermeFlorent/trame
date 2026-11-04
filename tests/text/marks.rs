use trame::geometry::Point;
use trame::render::Scene;
use trame::term::{Marks, Terminal};
use trame::text::{CellText, TextSystem};
use trame::theme::{Appearance, Rgba, Theme};
use trame::ui::terminal::{TerminalPaint, draw_terminal};

use crate::{cell, system};

const ORIGIN: Point = Point::new(20.0, 20.0);

type Placed = ([f32; 4], [f32; 4]);

fn marked(ch: char, marks: &[char]) -> CellText {
    CellText {
        marks: marks
            .iter()
            .fold(Marks::default(), |marks, &mark| marks.with(mark)),
        ..cell(ch, false, false, 1)
    }
}

fn drawn(text: &mut TextSystem, cell: CellText) -> Vec<Placed> {
    let mut scene = Scene::new(100.0, 100.0);
    text.draw_cell(&mut scene, cell, ORIGIN, Rgba::new(1.0, 1.0, 1.0, 1.0));
    scene
        .quads()
        .iter()
        .map(|quad| (quad.rect, quad.uv))
        .collect()
}

fn painted(text: &mut TextSystem, input: &str) -> usize {
    let mut terminal = Terminal::new(2, 10, 0);
    terminal.feed(input.as_bytes());
    let mut scene = Scene::new(400.0, 200.0);
    let paint = TerminalPaint {
        screen: terminal.screen(),
        origin: Point::new(0.0, 0.0),
        focused: false,
        scroll: 0.0,
        selected: &|_, _| false,
    };
    draw_terminal(&mut scene, text, &Theme::new(Appearance::Dark), &paint);
    scene.quads().len()
}

#[test]
fn a_composable_mark_draws_the_precomposed_glyph() {
    let mut text = system();
    let precomposed = drawn(&mut text, cell('\u{e9}', false, false, 1));
    assert_eq!(precomposed.len(), 1);
    assert_eq!(drawn(&mut text, marked('e', &['\u{301}'])), precomposed);
}

#[test]
fn other_marks_are_drawn_over_their_base() {
    let mut text = system();
    let width = text.cell_metrics().width;
    let base = drawn(&mut text, cell('x', false, false, 1));
    let glyphs = drawn(&mut text, marked('x', &['\u{301}']));
    assert_eq!(glyphs.len(), 2);
    assert_eq!(glyphs[0], base[0]);
    let (mark, _) = glyphs[1];
    let center = mark[0] + mark[2] / 2.0 - ORIGIN.x;
    assert!(
        center > 0.0 && center < width,
        "the mark is outside its cell at {center}"
    );
    assert!(mark[1] < base[0].0[1], "the mark is not above its base");
}

#[test]
fn joiners_and_selectors_add_no_glyph() {
    let mut text = system();
    let plain = drawn(&mut text, cell('a', false, false, 1));
    for mark in ['\u{200D}', '\u{FE0F}'] {
        assert_eq!(drawn(&mut text, marked('a', &[mark])), plain);
    }
}

#[test]
fn a_mark_on_a_space_is_painted() {
    let mut text = system();
    assert_eq!(painted(&mut text, " \u{301}"), painted(&mut text, " ") + 1);
}
