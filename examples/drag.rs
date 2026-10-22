use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use trame::geometry::Point;
use trame::render::{Gpu, OFFSCREEN_FORMAT, Renderer, Scene, render_to_rgba};
use trame::session::SessionId;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::theme::{Appearance, Theme};
use trame::ui::drop::DragView;
use trame::ui::metrics::TERMINAL_FONT_SIZE;
use trame::ui::{FrameInput, FrameLayout, FrameSources, FrameView, Motion, UiMetrics, draw_frame};
use trame::workspace::{Edge, Workspace};

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 760.0;
const LABELS: [&str; 4] = ["zsh", "cargo watch", "git log", "htop"];
const BODY_POINT: (f32, f32) = (0.92, 0.5);
const TAB_SHIFT: f32 = 0.15;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let output = PathBuf::from(args.next().ok_or("usage: drag <dir> [scale]")?);
    let scale: f32 = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1.0);
    let metrics = UiMetrics::new(scale);
    let size = ((WIDTH * scale).round(), (HEIGHT * scale).round());
    let mut text = TextSystem::new(TERMINAL_FONT_SIZE * scale, DEFAULT_LINE_HEIGHT)?;
    let workspace = build();
    let title = |id: SessionId| LABELS[(id.0 as usize - 1) % LABELS.len()].to_string();
    let input = FrameInput {
        workspace: &workspace,
        width: size.0,
        height: size.1,
        metrics,
        maximized: false,
        title: &title,
    };
    let frame = FrameLayout::compute(&mut text, &input);
    let gpu = Gpu::headless()?;
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, text.atlas_mut());
    let instance = workspace.active_instance().ok_or("no instance")?.id;
    let sessions = HashMap::new();
    let motion = Motion::default();
    for (name, appearance, pointer) in pointers(&frame) {
        let theme = Theme::new(appearance);
        let drag = DragView {
            label: title(SessionId(2)),
            pointer,
            mark: frame.drop_at(pointer, instance).map(|spot| spot.mark),
        };
        let view = FrameView {
            window_focused: true,
            ..FrameView::default()
        };
        let mut scene = Scene::new(size.0, size.1);
        let sources = FrameSources {
            theme: &theme,
            frame: &frame,
            view: &view,
            sessions: &sessions,
            motion: &motion,
            drag: Some(&drag),
        };
        draw_frame(&mut scene, &mut text, &sources);
        text.end_frame();
        renderer.sync_atlas(&gpu, text.atlas_mut());
        let pixels = render_to_rgba(&gpu, &mut renderer, &scene, theme.terminal)?;
        let path = output.join(format!("drag-{name}-{scale}.png"));
        write_png(&path, size.0 as u32, size.1 as u32, &pixels)?;
    }
    Ok(())
}

fn build() -> Workspace {
    let mut workspace = Workspace::new();
    workspace.add_instance(Some("trame".into()), SessionId(1));
    workspace.add_tab(SessionId(2));
    workspace.split_active(SessionId(3), Edge::Right);
    workspace.add_instance(Some("serveur de dev".into()), SessionId(4));
    workspace.active = 0;
    workspace
}

fn pointers(frame: &FrameLayout) -> Vec<(&'static str, Appearance, Point)> {
    let body = frame.panes[0].body;
    let split = Point::new(
        body.x + body.width * BODY_POINT.0,
        body.y + body.height * BODY_POINT.1,
    );
    let strip = &frame.panes[1].strip;
    let tab = strip.tabs[0].rect;
    let insert = Point::new(tab.right() - tab.width * TAB_SHIFT, tab.center().y);
    vec![
        ("split", Appearance::Dark, split),
        ("tabs", Appearance::Light, insert),
        ("center", Appearance::Dark, frame.panes[1].body.center()),
    ]
}

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(pixels)?;
    Ok(())
}
