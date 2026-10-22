mod common;

use std::collections::HashMap;
use std::error::Error;

use common::{Args, Studio};
use trame::geometry::Point;
use trame::render::Scene;
use trame::session::SessionId;
use trame::text::TextSystem;
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
    let args = Args::parse("usage: drag <dir> [scale]")?;
    let scale = args.scale;
    let mut studio = Studio::new(TERMINAL_FONT_SIZE * scale)?;
    let workspace = build();
    let frame = layout(&mut studio.text, &workspace, scale);
    let instance = workspace.active_instance().ok_or("no instance")?.id;
    for (name, appearance, pointer) in pointers(&frame) {
        let theme = Theme::new(appearance);
        let drag = DragView {
            label: title(SessionId(2)),
            pointer,
            mark: frame.drop_at(pointer, instance).map(|spot| spot.mark),
            ghost: true,
        };
        let scene = paint(&mut studio.text, &frame, &theme, &drag);
        let path = args.output.join(format!("drag-{name}-{scale}.png"));
        studio.save(&scene, theme.terminal, &path)?;
    }
    Ok(())
}

fn title(id: SessionId) -> String {
    LABELS[(id.0 as usize - 1) % LABELS.len()].to_string()
}

fn layout(text: &mut TextSystem, workspace: &Workspace, scale: f32) -> FrameLayout {
    let input = FrameInput {
        workspace,
        width: (WIDTH * scale).round(),
        height: (HEIGHT * scale).round(),
        metrics: UiMetrics::new(scale),
        maximized: false,
        title: &title,
    };
    FrameLayout::compute(text, &input)
}

fn paint(text: &mut TextSystem, frame: &FrameLayout, theme: &Theme, drag: &DragView) -> Scene {
    let view = FrameView {
        window_focused: true,
        ..FrameView::default()
    };
    let sources = FrameSources {
        theme,
        frame,
        view: &view,
        sessions: &HashMap::new(),
        motion: &Motion::default(),
        drag: Some(drag),
        loader: None,
    };
    let mut scene = Scene::new(frame.bounds.width, frame.bounds.height);
    draw_frame(&mut scene, text, &sources);
    scene
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
