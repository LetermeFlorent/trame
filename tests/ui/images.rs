use std::sync::Arc;

use trame::geometry::Point;
use trame::render::Scene;
use trame::term::Terminal;
use trame::term::graphics::{Image, ImageTile, Layer};
use trame::text::CellMetrics;
use trame::theme::{Appearance, Theme};
use trame::ui::terminal::{ScreenCopy, TerminalPaint, draw_terminal, draw_tiles};

use crate::text_system;

const IMAGE_KIND: f32 = 6.0;
const RED_2X2: &str = "/wAA//8AAP//AAD//wAA/w==";

fn metrics() -> CellMetrics {
    CellMetrics {
        font_size: 10.0,
        width: 8.0,
        height: 16.0,
        baseline: 12.0,
        underline_position: 14.0,
        underline_thickness: 1.0,
        strike_position: 8.0,
        strike_thickness: 1.0,
    }
}

fn tile(layer: Layer) -> ImageTile {
    ImageTile {
        image: Image {
            serial: 4,
            width: 2,
            height: 2,
            pixels: Arc::from(vec![255; 16]),
        },
        uv: [0.0, 0.5, 1.0, 1.0],
        row: 2,
        col: 3,
        cols: 2.0,
        rows: 1.5,
        offset: (1, 2),
        layer,
    }
}

fn shown(input: &str) -> Terminal {
    let mut terminal = Terminal::new(4, 10, 10);
    terminal.feed(format!("\x1b_Ga=T,f=32,s=2,v=2,c=2,r=2,q=2;{RED_2X2}\x1b\\{input}").as_bytes());
    terminal
}

#[test]
fn tiles_become_image_quads_at_their_cells() {
    let mut scene = Scene::new(400.0, 400.0);
    let tiles = [tile(Layer::UnderText), tile(Layer::OverText)];
    draw_tiles(
        &mut scene,
        &metrics(),
        Point::new(10.0, 20.0),
        &tiles,
        Layer::OverText,
    );
    let [quad] = scene.quads() else {
        panic!("one quad: {:?}", scene.quads());
    };
    assert_eq!(quad.params[0], IMAGE_KIND);
    assert_eq!(
        quad.rect,
        [10.0 + 24.0 + 1.0, 20.0 + 32.0 + 2.0, 16.0, 24.0]
    );
    assert_eq!(quad.uv, [0.0, 0.5, 1.0, 1.0]);
    assert_eq!(scene.images()[0].serial, 4);
}

#[test]
fn a_copy_keeps_tiles_on_their_rows_while_scrolling() {
    let terminal = shown("");
    for scroll in [0.0, 0.5] {
        let copy = ScreenCopy::of(terminal.screen(), scroll);
        let rows: Vec<isize> = copy.tiles.iter().map(|tile| tile.row).collect();
        assert_eq!(rows, vec![0], "scroll {scroll}");
    }
}

#[test]
fn a_selection_over_an_image_is_drawn_after_it() {
    let terminal = shown("");
    let mut scene = Scene::new(400.0, 200.0);
    let paint = TerminalPaint {
        screen: &ScreenCopy::of(terminal.screen(), 0.0),
        origin: Point::new(0.0, 0.0),
        focused: false,
        scroll: 0.0,
        selected: &|row, col| row == 0 && col == 0,
        links: &[],
    };
    let mut text = text_system();
    draw_terminal(&mut scene, &mut text, &Theme::new(Appearance::Dark), &paint);
    let image = scene.images()[0].quad as usize;
    let cell = text.cell_metrics();
    let fill = [0.0, 0.0, cell.width, cell.height];
    let after = &scene.quads()[image + 1..];
    assert!(
        after
            .iter()
            .any(|quad| quad.rect == fill && quad.params[0] == 0.0)
    );
}
