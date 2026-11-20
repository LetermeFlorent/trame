mod color_atlas;
mod offscreen;
mod present;
mod recovery_loop;
mod software;

use super::recovery::{RecoveryPolicy, recover_session};
use super::session::Session;
use trame::geometry::Rect;
use trame::render::{Atlas, AtlasKind, Gpu, OFFSCREEN_FORMAT, Renderer, Scene, render_in_format};
use trame::theme::Rgba;

pub(super) fn open_headless(atlas: &mut Atlas, software: bool) -> Result<Session<()>, String> {
    let gpu = if software {
        Gpu::headless_software().map_err(|error| error.to_string())?
    } else {
        Gpu::headless().map_err(|error| error.to_string())?
    };
    let renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, atlas);
    Ok(Session {
        gpu,
        surface: (),
        format: OFFSCREEN_FORMAT,
        renderer,
        software,
    })
}

fn draw_glyph(session: &mut Session<()>, atlas: &mut Atlas, color: Rgba) -> Vec<u8> {
    let region = atlas
        .insert(AtlasKind::Mask, 2, 2, vec![255; 4])
        .expect("glyph fits");
    assert!(!atlas.finish_frame());
    session.renderer.sync_atlas(&session.gpu, atlas);
    let mut scene = Scene::new(4.0, 4.0);
    scene.fill(
        Rect::new(0.0, 0.0, 4.0, 4.0),
        Rgba::new(0.0, 0.0, 0.0, 1.0),
        0.0,
    );
    scene.glyph(region, 0.0, 0.0, color);
    render_in_format(
        &session.gpu,
        &mut session.renderer,
        &scene,
        Rgba::new(0.0, 0.0, 0.0, 1.0),
        session.format,
    )
    .expect("offscreen render")
}

#[test]
fn lost_device_recovery_recreates_session_and_invalidates_atlas() {
    let mut atlas = Atlas::new();
    let mut session = Some(open_headless(&mut atlas, false).expect("initial session"));
    let expected = draw_glyph(
        session.as_mut().unwrap(),
        &mut atlas,
        Rgba::new(1.0, 0.0, 0.0, 1.0),
    );
    assert!(expected[0] > 0);
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
        &mut atlas,
        |atlas, _| open_headless(atlas, false),
        &mut |line| logs.push(line.to_owned()),
    );
    assert!(ok);
    assert!(
        logs.iter().any(|line| line.contains("recovered")),
        "{logs:?}"
    );
    let active = session.as_mut().unwrap();
    assert!(!active.gpu.is_lost());
    assert!(atlas.finish_frame());
    atlas.clear();
    let after = draw_glyph(active, &mut atlas, Rgba::new(1.0, 0.0, 0.0, 1.0));
    assert_eq!(after, expected);
}

#[test]
fn recovery_falls_back_to_software_adapter_and_renders() {
    let mut atlas = Atlas::new();
    let mut session = Some(open_headless(&mut atlas, false).expect("initial session"));
    let mut policy = RecoveryPolicy::default();
    let mut fallback_logs = Vec::new();
    let mut logs = Vec::new();
    let ok = recover_session(
        &mut session,
        &mut policy,
        &mut atlas,
        |atlas, _| {
            let gpu = Gpu::headless_with(wgpu::Backends::empty(), &mut |line| {
                fallback_logs.push(line.to_owned());
            })
            .map_err(|error| error.to_string())?;
            let renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, atlas);
            Ok::<_, String>(Session {
                gpu,
                surface: (),
                format: OFFSCREEN_FORMAT,
                renderer,
                software: true,
            })
        },
        &mut |line| logs.push(line.to_owned()),
    );
    logs.extend(fallback_logs);
    assert!(ok);
    let active = session.as_mut().unwrap();
    assert_eq!(active.gpu.adapter_info().device_type, wgpu::DeviceType::Cpu);
    assert!(
        logs.iter().any(|line| line.contains("software")),
        "{logs:?}"
    );
    assert!(atlas.finish_frame());
    atlas.clear();
    let pixels = draw_glyph(active, &mut atlas, Rgba::new(0.0, 1.0, 0.0, 1.0));
    assert!(pixels[1] > 0);
}

#[test]
fn unrecoverable_device_loss_sets_failed_without_panicking() {
    let mut atlas = Atlas::new();
    let mut session: Option<Session<()>> = Some(open_headless(&mut atlas, false).expect("initial"));
    let mut policy = RecoveryPolicy::default();
    let mut logs = Vec::new();
    let ok = recover_session(
        &mut session,
        &mut policy,
        &mut atlas,
        |_, _| Err::<Session<()>, _>("no adapter available"),
        &mut |line| logs.push(line.to_owned()),
    );
    assert!(!ok);
    assert!(session.is_none());
    assert!(
        logs.iter().any(|line| line.contains("could not recover")),
        "{logs:?}"
    );
}
