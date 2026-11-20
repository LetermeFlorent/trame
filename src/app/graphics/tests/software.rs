use super::super::plan::{planned_attempts, try_connect};
use super::super::recovery::{RecoveryPolicy, recover_session};
use super::super::session::Session;
use super::open_headless;
use trame::render::Atlas;

fn open_software_fallback(
    atlas: &mut Atlas,
    force_software: bool,
    probed: &mut Vec<&'static str>,
) -> Result<Session<()>, String> {
    let enabled = wgpu::Backends::VULKAN | wgpu::Backends::DX12;
    let attempts = planned_attempts(wgpu::Backends::VULKAN, enabled, None, force_software);
    let mut step_logs = Vec::new();
    try_connect(
        &attempts,
        |attempt| {
            probed.push(attempt.label);
            if !attempt.software {
                return Err("hardware lost".to_owned());
            }
            let session = open_headless(atlas, true)?;
            Ok((session, "WARP".to_owned()))
        },
        &mut |line| step_logs.push(line.to_owned()),
    )
}

#[test]
fn software_fallback_preference_preserved_across_recoveries() {
    let mut atlas = Atlas::new();
    let mut session = Some(open_headless(&mut atlas, false).expect("initial session"));
    let mut policy = RecoveryPolicy {
        prefer_software: session.as_ref().unwrap().software,
        hardware_losses: 0,
    };
    let mut probed: Vec<&'static str> = Vec::new();
    let mut logs = Vec::new();

    let ok = recover_session(
        &mut session,
        &mut policy,
        &mut atlas,
        |atlas, force| open_software_fallback(atlas, force, &mut probed),
        &mut |line| logs.push(line.to_owned()),
    );
    assert!(ok);
    assert_eq!(probed, vec!["vulkan", "dx12", "software"]);
    assert!(policy.prefer_software);

    probed.clear();
    {
        let active = session.as_ref().unwrap();
        active.gpu.device.destroy();
        let _ = active.gpu.device.poll(wgpu::PollType::Poll);
        assert!(active.gpu.is_lost());
    }
    let ok = recover_session(
        &mut session,
        &mut policy,
        &mut atlas,
        |atlas, force| open_software_fallback(atlas, force, &mut probed),
        &mut |line| logs.push(line.to_owned()),
    );
    assert!(ok);
    assert_eq!(probed, vec!["software"]);
}
