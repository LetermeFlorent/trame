use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use trame::pty::{ShellCommand, default_shell};
use trame::render::{Gpu, OFFSCREEN_FORMAT, Renderer, Scene, render_to_rgba};
use trame::session::{Notifier, Session, SessionId, SessionOptions};
use trame::term::ReportedColors;
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::theme::{Appearance, Theme};
use trame::ui::metrics::TERMINAL_FONT_SIZE;
use trame::ui::{FrameInput, FrameLayout, FrameSources, FrameView, Hit, UiMetrics, draw_frame};
use trame::workspace::{Edge, Workspace};

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 760.0;
const STARTUP: Duration = Duration::from_millis(2500);
const SETTLE: Duration = Duration::from_millis(3000);
const COMMANDS: [&str; 3] = [
    "ls src\r",
    "git -c color.ui=always log --oneline --graph -12\r",
    "git status\r",
];

struct Scenario {
    workspace: Workspace,
    sessions: HashMap<SessionId, Session>,
    shell: ShellCommand,
    colors: ReportedColors,
    cwd: PathBuf,
}

impl Scenario {
    fn spawn(&mut self) -> Result<SessionId, Box<dyn Error>> {
        let id = SessionId(self.sessions.len() as u64 + 1);
        let options = SessionOptions {
            shell: &self.shell,
            cwd: Some(&self.cwd),
            rows: 24,
            cols: 80,
            colors: self.colors,
        };
        let notify: Notifier = Arc::new(|_| {});
        self.sessions
            .insert(id, Session::spawn(id, &options, notify)?);
        Ok(id)
    }

    fn label(&self, id: SessionId) -> String {
        self.sessions
            .get(&id)
            .map_or_else(String::new, Session::label)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let output = PathBuf::from(args.next().ok_or("usage: scenario <dir> [scale]")?);
    let scale: f32 = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1.0);
    let metrics = UiMetrics::new(scale);
    let size = ((WIDTH * scale).round(), (HEIGHT * scale).round());
    let mut text = TextSystem::new(TERMINAL_FONT_SIZE * scale, DEFAULT_LINE_HEIGHT)?;
    let mut scenario = build(Theme::new(Appearance::Dark).reported_colors())?;
    let frame = layout(&mut text, &scenario, metrics, size);
    let cell = text.cell_metrics();
    for pane in &frame.panes {
        let (rows, cols) = pane.grid_size(&cell);
        for tab in &pane.tabs {
            if let Some(session) = scenario.sessions.get_mut(tab) {
                session.resize(rows, cols);
            }
        }
    }
    thread::sleep(STARTUP);
    for (index, pane) in frame.panes.iter().enumerate() {
        let session = pane.active.and_then(|id| scenario.sessions.get(&id));
        if let Some(session) = session {
            session.write(COMMANDS[index % COMMANDS.len()]);
        }
    }
    thread::sleep(SETTLE);
    let gpu = Gpu::headless()?;
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, text.atlas_mut());
    for (name, appearance) in [("dark", Appearance::Dark), ("light", Appearance::Light)] {
        let theme = Theme::new(appearance);
        let frame = layout(&mut text, &scenario, metrics, size);
        let hover = frame.panes[0]
            .strip
            .tabs
            .last()
            .map(|tab| Hit::Tab(frame.panes[0].id, tab.id));
        let view = FrameView {
            hover,
            pressed: None,
            window_focused: true,
        };
        let mut scene = Scene::new(size.0, size.1);
        let sources = FrameSources {
            sessions: &scenario.sessions,
        };
        draw_frame(&mut scene, &mut text, &theme, &frame, &view, &sources);
        text.end_frame();
        renderer.sync_atlas(&gpu, text.atlas_mut());
        let pixels = render_to_rgba(&gpu, &mut renderer, &scene, theme.terminal);
        let path = output.join(format!("frame-{name}-{scale}.png"));
        write_png(&path, size.0 as u32, size.1 as u32, &pixels)?;
    }
    Ok(())
}

fn build(colors: ReportedColors) -> Result<Scenario, Box<dyn Error>> {
    let mut scenario = Scenario {
        workspace: Workspace::new(),
        sessions: HashMap::new(),
        shell: default_shell(),
        colors,
        cwd: env::current_dir()?,
    };
    let first = scenario.spawn()?;
    scenario.workspace.add_instance(Some("trame".into()), first);
    let second = scenario.spawn()?;
    scenario.workspace.add_tab(second);
    let third = scenario.spawn()?;
    scenario.workspace.split_active(third, Edge::Right);
    let fourth = scenario.spawn()?;
    scenario
        .workspace
        .add_instance(Some("serveur de dev".into()), fourth);
    scenario.workspace.active = 0;
    Ok(scenario)
}

fn layout(
    text: &mut TextSystem,
    scenario: &Scenario,
    metrics: UiMetrics,
    size: (f32, f32),
) -> FrameLayout {
    let title = |id: SessionId| scenario.label(id);
    let input = FrameInput {
        workspace: &scenario.workspace,
        width: size.0,
        height: size.1,
        metrics,
        maximized: false,
        title: &title,
    };
    FrameLayout::compute(text, &input)
}

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(pixels)?;
    Ok(())
}
