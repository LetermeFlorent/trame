#[path = "../common/mod.rs"]
mod common;
mod sample;

use std::error::Error;

use common::{Args, Studio};
use sample::{LINES, Sample};
use trame::geometry::{Point, Rect};
use trame::render::Scene;
use trame::text::{CellText, FontRole, REGULAR, SEMIBOLD, TextLayout, TextStyle, TextSystem};
use trame::theme::Rgba;

const WIDTH: f32 = 760.0;
const HEIGHT: f32 = 420.0;
const TERMINAL_SIZE: f32 = 14.0;
const INTERFACE_SIZE: f32 = 12.5;
const BACKGROUND: Rgba = Rgba::hex(0x1f2329);
const SURFACE: Rgba = Rgba::hex(0x2b2f36);
const BORDER: Rgba = Rgba::hex(0x3b414a);
const TEXT: Rgba = Rgba::hex(0xdce0e5);
const MUTED: Rgba = Rgba::hex(0x878e98);
const ACCENT: Rgba = Rgba::hex(0x74ade8);

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse("usage: preview <output.png> [scale]")?;
    let scale = args.scale;
    let mut studio = Studio::new(TERMINAL_SIZE * scale)?;
    let mut scene = Scene::new((WIDTH * scale).round(), (HEIGHT * scale).round());
    draw(&mut scene, &mut studio.text, scale);
    studio.save(&scene, BACKGROUND, &args.output)
}

fn draw(scene: &mut Scene, text: &mut TextSystem, scale: f32) {
    let bar = Rect::new(0.0, 0.0, scene.width(), (30.0 * scale).round());
    scene.fill(bar, SURFACE, 0.0);
    draw_tabs(scene, text, scale);
    let cell = text.cell_metrics();
    let origin = Point::new(16.0 * scale, bar.bottom() + 14.0 * scale);
    for (row, line) in LINES.iter().enumerate() {
        let y = origin.y + row as f32 * cell.height;
        draw_line(scene, text, line, Point::new(origin.x, y));
    }
    let cursor_y = origin.y + LINES.len() as f32 * cell.height;
    scene.fill(
        Rect::new(origin.x, cursor_y, cell.width, cell.height),
        ACCENT,
        0.0,
    );
}

fn draw_tabs(scene: &mut Scene, text: &mut TextSystem, scale: f32) {
    let tab = Rect::new(12.0 * scale, 5.0 * scale, 150.0 * scale, 20.0 * scale).snapped();
    scene.fill(tab, BACKGROUND, 5.0 * scale);
    scene.border(tab, BORDER, 5.0 * scale, scale);
    let style = TextStyle {
        role: FontRole::Interface,
        size: INTERFACE_SIZE * scale,
        weight: REGULAR,
    };
    let title = text.layout("pwsh  ~/Projets/trame", style);
    let pen = baseline(&title, tab.inset_by(10.0 * scale, 0.0));
    text.draw_layout(scene, &title, pen, TEXT);
    let strong = TextStyle {
        weight: SEMIBOLD,
        ..style
    };
    let second = Rect::new(172.0 * scale, 5.0 * scale, 120.0 * scale, 20.0 * scale);
    let caption = text.layout("Instances", strong);
    let pen = baseline(&caption, second.inset_by(10.0 * scale, 0.0));
    text.draw_layout(scene, &caption, pen, MUTED);
}

fn baseline(layout: &TextLayout, rect: Rect) -> Point {
    Point::new(rect.x, (rect.center().y + layout.cap_height / 2.0).round())
}

fn draw_line(scene: &mut Scene, text: &mut TextSystem, sample: &Sample, origin: Point) {
    let &(line, bold, italic, color) = sample;
    let cell = text.cell_metrics();
    let mut column = 0.0;
    for ch in line.chars() {
        let columns = unicode_columns(ch);
        let position = Point::new(origin.x + column * cell.width, origin.y);
        let value = CellText {
            ch,
            bold,
            italic,
            columns,
        };
        text.draw_cell(scene, value, position, color);
        column += f32::from(columns);
    }
}

fn unicode_columns(ch: char) -> u8 {
    use unicode_width::UnicodeWidthChar;
    ch.width().map_or(1, |width| width.clamp(1, 2) as u8)
}
