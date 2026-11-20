use super::super::recovery::GraphicsState;
use super::super::session::FrameOutcome;
use super::open_headless;
use trame::render::{Atlas, ReadbackError};

#[test]
fn failed_graphics_does_not_reprobe_backends_on_redraw_or_render_image() {
    let mut atlas = Atlas::new();
    let session = open_headless(&mut atlas, false).expect("session");
    let mut state = GraphicsState::new(session);
    let mut open_calls = 0;
    let mut logs = Vec::new();

    let first = state.present_with(
        &mut atlas,
        |_, _| FrameOutcome::Lost,
        |_, _| {
            open_calls += 1;
            Err::<_, &str>("no adapter")
        },
        &mut |line| logs.push(line.to_owned()),
    );
    assert!(!first);
    assert!(state.failed);
    assert_eq!(open_calls, 1);

    for _ in 0..3 {
        let drawn = state.present_with(
            &mut atlas,
            |_, _| panic!("draw must not run after failure"),
            |_, _| {
                open_calls += 1;
                Err::<_, &str>("reprobed")
            },
            &mut |line| logs.push(line.to_owned()),
        );
        assert!(!drawn);
    }
    let image = state.render_image_with(
        &mut atlas,
        |_, _| panic!("render must not run after failure"),
        |_, _| {
            open_calls += 1;
            Err::<_, &str>("reprobed")
        },
        &mut |line| logs.push(line.to_owned()),
    );
    assert!(matches!(image, Err(ReadbackError::Lost)));
    assert_eq!(open_calls, 1);
}

#[test]
fn repeated_consecutive_hardware_losses_escalate_to_software() {
    let mut atlas = Atlas::new();
    let session = open_headless(&mut atlas, false).expect("session");
    let mut state = GraphicsState::new(session);
    let mut flags = Vec::new();
    let mut logs = Vec::new();

    assert!(!state.present_with(
        &mut atlas,
        |_, _| FrameOutcome::Lost,
        |atlas, prefer| {
            flags.push(prefer);
            open_headless(atlas, prefer)
        },
        &mut |line| logs.push(line.to_owned()),
    ));
    assert!(!state.present_with(
        &mut atlas,
        |_, _| FrameOutcome::Skipped,
        |atlas, prefer| {
            flags.push(prefer);
            open_headless(atlas, prefer)
        },
        &mut |line| logs.push(line.to_owned()),
    ));
    assert!(!state.present_with(
        &mut atlas,
        |_, _| FrameOutcome::Lost,
        |atlas, prefer| {
            flags.push(prefer);
            open_headless(atlas, prefer)
        },
        &mut |line| logs.push(line.to_owned()),
    ));
    assert_eq!(flags, vec![false, true]);
    assert!(state.policy.prefer_software);
}

#[test]
fn successful_present_resets_consecutive_hardware_loss_count() {
    let mut atlas = Atlas::new();
    let session = open_headless(&mut atlas, false).expect("session");
    let mut state = GraphicsState::new(session);
    let mut flags = Vec::new();
    let mut logs = Vec::new();

    assert!(!state.present_with(
        &mut atlas,
        |_, _| FrameOutcome::Lost,
        |atlas, prefer| {
            flags.push(prefer);
            open_headless(atlas, prefer)
        },
        &mut |line| logs.push(line.to_owned()),
    ));
    assert!(state.present_with(
        &mut atlas,
        |_, _| FrameOutcome::Presented,
        |_, _| -> Result<_, &str> { panic!("must not recover on Presented") },
        &mut |line| logs.push(line.to_owned()),
    ));
    assert!(!state.present_with(
        &mut atlas,
        |_, _| FrameOutcome::Lost,
        |atlas, prefer| {
            flags.push(prefer);
            open_headless(atlas, prefer)
        },
        &mut |line| logs.push(line.to_owned()),
    ));
    assert!(!state.present_with(
        &mut atlas,
        |_, _| FrameOutcome::Lost,
        |atlas, prefer| {
            flags.push(prefer);
            open_headless(atlas, prefer)
        },
        &mut |line| logs.push(line.to_owned()),
    ));
    assert_eq!(flags, vec![false, false, true]);
}
