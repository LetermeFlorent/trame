use super::super::recovery::{RecoveryPolicy, recover_session};
use super::super::session::Session;
use super::open_headless;
use trame::geometry::Point;
use trame::render::{OFFSCREEN_FORMAT, Scene, render_in_format};
use trame::text::{CellText, DEFAULT_LINE_HEIGHT, ImageKey, TextSystem};
use trame::theme::Rgba;

fn paint_glyph_and_icon(session: &mut Session<()>, text: &mut TextSystem) -> Vec<u8> {
    let mut scene = Scene::new(16.0, 16.0);
    for _ in 0..2 {
        scene.reset(16.0, 16.0);
        text.draw_cell(
            &mut scene,
            CellText {
                ch: 'A',
                marks: Default::default(),
                bold: false,
                italic: false,
                columns: 1,
            },
            Point::new(0.0, 0.0),
            Rgba::new(1.0, 1.0, 1.0, 1.0),
        );
        let key = ImageKey {
            id: 7,
            width: 4,
            height: 4,
        };
        let region = text
            .image(key, || [0, 255, 128, 255].repeat(16))
            .expect("color icon fits");
        scene.glyph(region, 8.0, 8.0, Rgba::new(1.0, 1.0, 1.0, 1.0));
        if !text.end_frame() {
            break;
        }
    }
    session.renderer.sync_atlas(&session.gpu, text.atlas_mut());
    render_in_format(
        &session.gpu,
        &mut session.renderer,
        &scene,
        Rgba::new(0.0, 0.0, 0.0, 1.0),
        OFFSCREEN_FORMAT,
    )
    .expect("offscreen render")
}

#[test]
fn text_system_clears_color_icon_and_glyph_atlases_on_recover() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("monospace font");
    let mut session = Some(open_headless(text.atlas_mut(), false).expect("initial session"));
    let before = paint_glyph_and_icon(session.as_mut().unwrap(), &mut text);
    assert!(before.chunks(4).any(|px| px[1] == 255 && px[2] == 128));
    {
        let active = session.as_ref().unwrap();
        active.gpu.device.destroy();
        let _ = active.gpu.device.poll(wgpu::PollType::Poll);
        assert!(active.gpu.is_lost());
    }
    let mut policy = RecoveryPolicy::default();
    let mut logs = Vec::new();
    let ok = recover_session(
        &mut session,
        &mut policy,
        text.atlas_mut(),
        |atlas, _| open_headless(atlas, false),
        &mut |line| logs.push(line.to_owned()),
    );
    assert!(ok);
    let after = paint_glyph_and_icon(session.as_mut().unwrap(), &mut text);
    assert_eq!(after, before);
}
