use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use trame::desk::{Sessions, Waiting};
use trame::files::IconStore;
use trame::render::Scene;
use trame::session::pty::ShellCommand;
use trame::session::{Session, SessionEvent, SessionId, SessionOptions};
use trame::term::ReportedColors;
use trame::text::TextSystem;
use trame::theme::{Appearance, Theme};
use trame::ui::terminal::links::LinkCache;
use trame::ui::{FrameLayout, FrameSources, FrameView, Motion, draw_frame};
use trame::workspace::Workspace;

use super::TempDir;
use crate::{HEIGHT, WIDTH, frame, text_system};

pub(super) fn silent_session(id: SessionId) -> Session {
    let (tx, rx) = mpsc::channel();
    let (prog, a0, a1) = if cfg!(windows) {
        ("cmd.exe", "/C", "rem")
    } else {
        ("/bin/sh", "-c", ":")
    };
    let shell = ShellCommand {
        program: PathBuf::from(prog),
        args: [OsString::from(a0), OsString::from(a1)].into(),
        env: Vec::new(),
    };
    let opts = SessionOptions {
        shell: &shell,
        cwd: None,
        rows: 8,
        cols: 40,
        history: 0,
        colors: ReportedColors::default(),
        transcript: b"",
    };
    let notify = Arc::new(move |ev| {
        let _ = tx.send(ev);
    });
    let session = Session::spawn(id, &opts, notify).expect("spawn");
    while let Ok(ev) = rx.recv_timeout(Duration::from_secs(5)) {
        session.take_output();
        if ev == SessionEvent::Exited(id) {
            break;
        }
    }
    while rx.recv_timeout(Duration::from_millis(30)).is_ok() {
        session.take_output();
    }
    session.terminal().feed(b"\x1b[?25l\x1b[2J\x1b[Ha.png");
    session
}

pub(super) fn underline_count(
    text: &mut TextSystem,
    layout: &FrameLayout,
    sessions: &HashMap<SessionId, Session>,
    cwd: Option<&dyn Fn(SessionId) -> Option<PathBuf>>,
) -> usize {
    let mut scene = Scene::new(WIDTH, HEIGHT);
    let cache = RefCell::new(LinkCache::default());
    let sources = FrameSources {
        theme: &Theme::new(Appearance::Dark),
        frame: layout,
        view: &FrameView::default(),
        sessions,
        motion: &Motion::default(),
        drag: None,
        loader: None,
        icons: &IconStore::default(),
        links: Some(&cache),
        cwd,
        picture: None,
        preview: None,
    };
    draw_frame(&mut scene, text, &sources);
    let pane = layout.panes[0].terminal;
    let m = text.cell_metrics();
    let y = pane.y + m.underline_position;
    let t = m.underline_thickness.max(1.0);
    scene
        .quads()
        .iter()
        .filter(|q| {
            q.params[0] == 0.0
                && q.rect[2] == m.width
                && q.rect[3] == t
                && (q.rect[1] - y).abs() < 0.01
                && q.rect[0] >= pane.x
        })
        .count()
}

#[test]
fn draw_frame_underlines_relative_image_from_session_cwd_without_osc7() {
    let dir = TempDir::new("draw-cwd");
    fs::write(dir.0.join("a.png"), b"png").expect("png");
    let mut sessions = Sessions::new(Arc::new(|_| {}));
    let id = sessions.reserve();
    sessions.defer(id, Some(dir.0.clone()), Waiting { transcript: None });
    let session = silent_session(id);
    assert!(session.cwd().is_none());
    let mut map = HashMap::from([(id, session)]);
    let mut ws = Workspace::new();
    ws.add_instance(None, id);
    let mut text = text_system();
    let layout = frame(&mut text, &ws);
    assert_eq!(underline_count(&mut text, &layout, &map, None), 0);
    assert_eq!(
        underline_count(&mut text, &layout, &map, Some(&|sid| sessions.cwd(sid))),
        5
    );
    map.clear();
}
