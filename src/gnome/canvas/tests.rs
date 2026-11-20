use gtk::cairo;
use trame::files::IconStore;
use trame::geometry::{Point, Rect};
use trame::locale::LanguageChoice;
use trame::preferences::Preferences;
use trame::render::{Atlas, AtlasKind, Scene};
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};
use trame::theme::{Appearance, Rgba, Theme};
use trame::ui::chrome::{ChromeInput, draw_settings};
use trame::ui::frame::{SettingControl, SettingsRequest, SettingsTab};
use trame::ui::{FrameInput, FrameLayout, FrameView, Motion, UiMetrics};
use trame::workspace::Workspace;

use super::ScenePainter;

fn pixel_at(surface: &mut cairo::ImageSurface, x: usize, y: usize) -> [u8; 4] {
    surface.flush();
    let stride = surface.stride() as usize;
    let data = surface.data().unwrap();
    let offset = y * stride + x * 4;
    let word = u32::from_ne_bytes(data[offset..offset + 4].try_into().unwrap());
    [
        ((word >> 16) & 0xff) as u8,
        ((word >> 8) & 0xff) as u8,
        (word & 0xff) as u8,
        ((word >> 24) & 0xff) as u8,
    ]
}

fn base_frame(text: &mut TextSystem, workspace: &Workspace) -> FrameLayout {
    let strings = LanguageChoice::English.resolve().strings();
    let title = |id| format!("tab-{id:?}");
    let input = FrameInput {
        workspace,
        width: 900.0,
        height: 700.0,
        metrics: UiMetrics::new(1.0),
        maximized: false,
        fullscreen: false,
        title: &title,
        folder: None,
        update: None,
        unsaved: false,
        update_failed: false,
        strings,
    };
    FrameLayout::compute(text, &input)
}

#[test]
fn uploads_and_quads_paint_clipped_pixels() {
    let mut atlas = Atlas::new();
    let mask = atlas.insert(AtlasKind::Mask, 4, 4, vec![255; 16]).unwrap();
    let icon = atlas
        .insert(AtlasKind::Color, 2, 2, [0_u8, 200, 100, 255].repeat(4))
        .unwrap();
    let mut painter = ScenePainter::default();
    painter.sync_atlas(&mut atlas);
    let mut scene = Scene::new(80.0, 60.0);
    scene.push_clip(Rect::new(0.0, 0.0, 40.0, 50.0));
    scene.fill(Rect::new(10.0, 10.0, 50.0, 20.0), Rgba::hex(0xff0000), 2.0);
    scene.border(
        Rect::new(2.0, 2.0, 10.0, 10.0),
        Rgba::hex(0x00ff00),
        1.0,
        2.0,
    );
    scene.line(
        Point::new(5.0, 35.0),
        Point::new(25.0, 35.0),
        3.0,
        Rgba::WHITE,
    );
    scene.glyph(mask, 12.0, 40.0, Rgba::hex(0x0000ff));
    scene.glyph(icon, 20.0, 40.0, Rgba::WHITE);
    scene.pop_clip();
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 80, 60).unwrap();
    let cr = cairo::Context::new(&surface).unwrap();
    painter.paint(&cr, scene.quads());
    drop(cr);
    assert_eq!(pixel_at(&mut surface, 20, 20), [255, 0, 0, 255]);
    assert_eq!(pixel_at(&mut surface, 50, 20), [0, 0, 0, 0]);
    assert_eq!(pixel_at(&mut surface, 13, 41), [0, 0, 255, 255]);
    assert_eq!(pixel_at(&mut surface, 21, 41), [0, 200, 100, 255]);
}

#[test]
fn connections_settings_renders_keep_remote_tasks_toggle() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).unwrap();
    let workspace = Workspace::default();
    let mut frame = base_frame(&mut text, &workspace);
    let prefs = Preferences {
        keep_remote_tasks: true,
        ..Default::default()
    };
    let strings = LanguageChoice::English.resolve().strings();
    let req = SettingsRequest {
        preferences: &prefs,
        strings,
        folder: None,
        tab: SettingsTab::Connections,
        hosts: None,
        shortcuts: None,
    };
    frame.place_settings(&mut text, &req);
    let page = frame.settings.as_ref().unwrap().connections().unwrap();
    let toggle = page.toggle.as_ref().unwrap();
    assert_eq!(toggle.control, SettingControl::KeepRemoteTasks);
    assert!(toggle.on);
    let mut scene = Scene::new(900.0, 700.0);
    let theme = Theme::new(Appearance::Dark);
    let view = FrameView::focused();
    let motion = Motion::default();
    let icons = IconStore::new(None);
    let chrome = ChromeInput {
        theme: &theme,
        frame: &frame,
        view: &view,
        motion: &motion,
        icons: &icons,
    };
    draw_settings(&mut scene, &mut text, &chrome);
    assert!(!scene.quads().is_empty());
}
