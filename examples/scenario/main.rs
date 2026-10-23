#[path = "../common/mod.rs"]
mod common;
mod shells;

use std::collections::HashMap;
use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};

use common::{Args, Studio};
use shells::{Scenario, build};
use trame::locale::Language;
use trame::render::Scene;
use trame::session::{Session, SessionId};
use trame::text::TextSystem;
use trame::theme::{Appearance, Theme};
use trame::ui::metrics::TERMINAL_FONT_SIZE;
use trame::ui::motion::DURATION;
use trame::ui::{
    FrameInput, FrameLayout, FrameSources, FrameView, Hit, Motion, UiMetrics, draw_frame,
};

const WIDTH: f32 = 1440.0;
const HEIGHT: f32 = 810.0;
const STARTUP: Duration = Duration::from_millis(2500);
const SETTLE: Duration = Duration::from_millis(3000);

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse("usage: scenario <dir> [scale]")?;
    let scale = args.scale;
    let mut studio = Studio::new(TERMINAL_FONT_SIZE * scale)?;
    let mut scenario = build(Theme::new(Appearance::Dark).reported_colors())?;
    let frame = layout(&mut studio.text, &scenario, scale);
    scenario.resize(&frame, &studio.text.cell_metrics());
    thread::sleep(STARTUP);
    scenario.type_commands(&frame);
    thread::sleep(SETTLE);
    for (name, appearance) in [("dark", Appearance::Dark), ("light", Appearance::Light)] {
        let theme = Theme::new(appearance);
        let frame = layout(&mut studio.text, &scenario, scale);
        let scene = paint(&mut studio.text, &frame, &theme, &scenario.sessions);
        let path = args.output.join(format!("frame-{name}-{scale}.png"));
        studio.save(&scene, theme.terminal, &path)?;
    }
    Ok(())
}

fn paint(
    text: &mut TextSystem,
    frame: &FrameLayout,
    theme: &Theme,
    sessions: &HashMap<SessionId, Session>,
) -> Scene {
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
    let start = Instant::now();
    let mut motion = Motion::new(start);
    motion.hover(hover);
    motion.tick(start + DURATION);
    let sources = FrameSources {
        theme,
        frame,
        view: &view,
        sessions,
        motion: &motion,
        drag: None,
        loader: None,
    };
    let mut scene = Scene::new(frame.bounds.width, frame.bounds.height);
    draw_frame(&mut scene, text, &sources);
    scene
}

fn layout(text: &mut TextSystem, scenario: &Scenario, scale: f32) -> FrameLayout {
    let title = |id: SessionId| scenario.label(id);
    let input = FrameInput {
        workspace: &scenario.workspace,
        width: (WIDTH * scale).round(),
        height: (HEIGHT * scale).round(),
        metrics: UiMetrics::new(scale),
        maximized: false,
        title: &title,
        folder: None,
        strings: Language::English.strings(),
    };
    FrameLayout::compute(text, &input)
}
