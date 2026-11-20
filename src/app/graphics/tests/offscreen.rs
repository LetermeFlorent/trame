use super::super::recovery::GraphicsState;
use super::open_headless;
use trame::render::{Atlas, ReadbackError, Scene};

#[test]
fn render_offscreen_returns_lost_when_needs_recovery_fires_after_render_in_format() {
    let mut atlas = Atlas::new();
    let mut session = open_headless(&mut atlas, false).expect("session");
    let scene = Scene::new(4.0, 4.0);
    let result = session.render_offscreen_with(&scene, &mut atlas, |gpu, _, _, _, _| {
        gpu.device.destroy();
        let _ = gpu.device.poll(wgpu::PollType::Poll);
        Ok(vec![1, 2, 3, 4])
    });
    assert!(matches!(result, Err(ReadbackError::Lost)), "{result:?}");
}

#[test]
fn render_image_returns_lost_when_needs_recovery_fires_after_render_offscreen() {
    let mut atlas = Atlas::new();
    let session = open_headless(&mut atlas, false).expect("session");
    let mut state = GraphicsState::new(session);
    let mut reopened = 0;
    let mut logs = Vec::new();
    let result = state.render_image_with(
        &mut atlas,
        |active, _| {
            active.gpu.device.destroy();
            let _ = active.gpu.device.poll(wgpu::PollType::Poll);
            Ok(vec![1, 2, 3, 4])
        },
        |atlas, prefer| {
            reopened += 1;
            open_headless(atlas, prefer)
        },
        &mut |line| logs.push(line.to_owned()),
    );
    assert!(matches!(result, Err(ReadbackError::Lost)), "{result:?}");
    assert_eq!(reopened, 1);
}
