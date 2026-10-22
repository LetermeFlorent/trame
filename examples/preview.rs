use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use trame::color::Rgba;
use trame::geometry::{Point, Rect};
use trame::render::{Gpu, OFFSCREEN_FORMAT, Renderer, Scene, render_to_rgba};
use trame::text::{
    CellText, DEFAULT_LINE_HEIGHT, FontRole, REGULAR, SEMIBOLD, TextStyle, TextSystem,
};

const WIDTH: f32 = 760.0;
const HEIGHT: f32 = 420.0;
const TERMINAL_SIZE: f32 = 14.0;
const INTERFACE_SIZE: f32 = 12.5;
const BACKGROUND: Rgba = Rgba::hex(0x1f2329);
const SURFACE: Rgba = Rgba::hex(0x2b2f36);
const TEXT: Rgba = Rgba::hex(0xdce0e5);
const MUTED: Rgba = Rgba::hex(0x878e98);
const ACCENT: Rgba = Rgba::hex(0x74ade8);

const LINES: &[(&str, bool, bool, Rgba)] = &[
    (
        "PS C:\\Projets\\trame> git status --short",
        false,
        false,
        TEXT,
    ),
    (
        " M src/text/system.rs   ?? examples/preview.rs",
        false,
        false,
        Rgba::hex(0xa1c181),
    ),
    (
        "Bold line: The quick brown fox jumps 0123456789",
        true,
        false,
        TEXT,
    ),
    (
        "Italic line: The quick brown fox jumps {}[]()",
        false,
        true,
        Rgba::hex(0xd07277),
    ),
    (
        "Bold italic: ->  =>  !=  <=  >=  ===  www",
        true,
        true,
        Rgba::hex(0xdec184),
    ),
    (
        "Unicode: café naïve Ελληνικά Кириллица → ✓ ✗ ★ ⚠",
        false,
        false,
        TEXT,
    ),
    ("Wide: 日本語 中文 한국어 😀 🚀 🎉", false, false, TEXT),
    (
        "Box: ┌──┬──┐ │ab│cd│ └──┴──┘ ▁▂▃▄▅▆▇█ ░▒▓",
        false,
        false,
        ACCENT,
    ),
];

fn main() -> Result<(), Box<dyn Error>> {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: preview <output.png> [scale]")?;
    let scale: f32 = env::args()
        .nth(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(1.0);
    let gpu = Gpu::headless()?;
    let mut text = TextSystem::new(TERMINAL_SIZE * scale, DEFAULT_LINE_HEIGHT)?;
    let mut scene = Scene::new((WIDTH * scale).round(), (HEIGHT * scale).round());
    draw(&mut scene, &mut text, scale);
    text.end_frame();
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, text.atlas_mut());
    renderer.sync_atlas(&gpu, text.atlas_mut());
    let pixels = render_to_rgba(&gpu, &mut renderer, &scene, BACKGROUND)?;
    write_png(
        &output,
        scene.width() as u32,
        scene.height() as u32,
        &pixels,
    )
}

fn draw(scene: &mut Scene, text: &mut TextSystem, scale: f32) {
    let bar = Rect::new(0.0, 0.0, scene.width(), (30.0 * scale).round());
    scene.fill(bar, SURFACE, 0.0);
    let tab = Rect::new(12.0 * scale, 5.0 * scale, 150.0 * scale, 20.0 * scale).snapped();
    scene.fill(tab, BACKGROUND, 5.0 * scale);
    scene.border(tab, Rgba::hex(0x3b414a), 5.0 * scale, scale);
    let style = TextStyle {
        role: FontRole::Interface,
        size: INTERFACE_SIZE * scale,
        weight: REGULAR,
        italic: false,
    };
    label(
        scene,
        text,
        "pwsh  ~/Projets/trame",
        tab.inset_by(10.0 * scale, 0.0),
        style,
        TEXT,
    );
    let strong = TextStyle {
        weight: SEMIBOLD,
        ..style
    };
    let second = Rect::new(172.0 * scale, 5.0 * scale, 120.0 * scale, 20.0 * scale);
    label(
        scene,
        text,
        "Instances",
        second.inset_by(10.0 * scale, 0.0),
        strong,
        MUTED,
    );
    let cell = text.cell_metrics();
    let origin = Point::new(16.0 * scale, bar.bottom() + 14.0 * scale);
    for (row, (line, bold, italic, color)) in LINES.iter().enumerate() {
        let y = origin.y + row as f32 * cell.height;
        draw_line(
            scene,
            text,
            line,
            *bold,
            *italic,
            Point::new(origin.x, y),
            *color,
        );
    }
    let cursor_y = origin.y + LINES.len() as f32 * cell.height;
    scene.fill(
        Rect::new(origin.x, cursor_y, cell.width, cell.height),
        ACCENT,
        0.0,
    );
}

fn label(
    scene: &mut Scene,
    text: &mut TextSystem,
    value: &str,
    rect: Rect,
    style: TextStyle,
    color: Rgba,
) {
    let layout = text.layout(value, style);
    let baseline = (rect.center().y + layout.cap_height / 2.0).round();
    text.draw_layout(scene, &layout, rect.x, baseline, color);
}

fn draw_line(
    scene: &mut Scene,
    text: &mut TextSystem,
    line: &str,
    bold: bool,
    italic: bool,
    origin: Point,
    color: Rgba,
) {
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

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(pixels)?;
    Ok(())
}
