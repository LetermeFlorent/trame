use super::super::plan::{Attempt, planned_attempts, try_connect};

#[test]
fn preferred_failure_falls_back_to_other_backend_with_log() {
    let enabled = wgpu::Backends::VULKAN | wgpu::Backends::DX12;
    let attempts = planned_attempts(wgpu::Backends::VULKAN, enabled, None, false);
    let mut seen = Vec::new();
    let mut logs = Vec::new();
    let result = try_connect(
        &attempts,
        |attempt| {
            seen.push((attempt.label, attempt.software));
            if attempt.backends == wgpu::Backends::VULKAN {
                Err("no usable adapter")
            } else {
                Ok(("dx12-ok", "AMD Radeon".to_owned()))
            }
        },
        &mut |line| logs.push(line.to_owned()),
    );
    assert_eq!(result, Ok("dx12-ok"));
    assert_eq!(seen, vec![("vulkan", false), ("dx12", false)]);
    assert_eq!(logs.len(), 2);
    assert!(
        logs[0].contains("vulkan") && logs[0].contains("dx12"),
        "{logs:?}"
    );
    assert!(
        logs[1].contains("dx12") && logs[1].contains("AMD Radeon"),
        "{logs:?}"
    );
}

#[test]
fn failing_all_hardware_backends_falls_back_to_software_with_logs() {
    let enabled = wgpu::Backends::VULKAN | wgpu::Backends::DX12;
    let attempts = planned_attempts(wgpu::Backends::VULKAN, enabled, None, false);
    let mut seen = Vec::new();
    let mut logs = Vec::new();
    let result = try_connect(
        &attempts,
        |attempt| {
            seen.push((attempt.label, attempt.software));
            if attempt.software {
                Ok(("warp-ok", "Microsoft Basic Render Driver".to_owned()))
            } else {
                Err("backend failed")
            }
        },
        &mut |line| logs.push(line.to_owned()),
    );
    assert_eq!(result, Ok("warp-ok"));
    assert_eq!(
        seen,
        vec![("vulkan", false), ("dx12", false), ("software", true)]
    );
    assert_eq!(logs.len(), 3);
    assert!(logs[0].contains("falling back to dx12"), "{logs:?}");
    assert!(logs[1].contains("falling back to software"), "{logs:?}");
    assert!(
        logs[2].contains("using the software graphics adapter")
            && logs[2].contains("Microsoft Basic Render Driver"),
        "{logs:?}"
    );
}

#[test]
fn failing_env_backend_falls_back_to_preferred_then_other_then_software() {
    let enabled = wgpu::Backends::VULKAN | wgpu::Backends::DX12;
    let attempts = planned_attempts(
        wgpu::Backends::VULKAN,
        enabled,
        Some(wgpu::Backends::GL),
        false,
    );
    let labels: Vec<&str> = attempts.iter().map(|attempt| attempt.label).collect();
    assert_eq!(labels, vec!["gl", "vulkan", "dx12", "software"]);
}

#[test]
fn forced_software_picks_software_adapter_with_log() {
    let enabled = wgpu::Backends::VULKAN | wgpu::Backends::DX12;
    let attempts = planned_attempts(wgpu::Backends::VULKAN, enabled, None, true);
    assert_eq!(
        attempts,
        vec![Attempt {
            backends: enabled,
            software: true,
            label: "software",
        }]
    );
    let mut logs = Vec::new();
    let result = try_connect(
        &attempts,
        |_| Ok::<_, &str>(("soft", "WARP".to_owned())),
        &mut |line| logs.push(line.to_owned()),
    );
    assert_eq!(result, Ok("soft"));
    assert_eq!(logs, vec!["using the software graphics adapter: WARP"]);
}

#[test]
fn failing_all_backends_including_software_returns_error_and_logs_each() {
    let enabled = wgpu::Backends::VULKAN | wgpu::Backends::DX12;
    let attempts = planned_attempts(wgpu::Backends::VULKAN, enabled, None, false);
    let mut logs = Vec::new();
    let result: Result<(), &str> = try_connect(&attempts, |_| Err("no adapter"), &mut |line| {
        logs.push(line.to_owned());
    });
    assert_eq!(result, Err("no adapter"));
    assert_eq!(logs.len(), 3);
    assert!(
        logs[0].contains("vulkan") && logs[0].contains("falling back to dx12"),
        "{logs:?}"
    );
    assert!(
        logs[1].contains("dx12") && logs[1].contains("falling back to software"),
        "{logs:?}"
    );
    assert!(
        logs[2].contains("software graphics backend failed"),
        "{logs:?}"
    );
}

#[test]
fn planned_attempts_skips_unenabled_preferred_backend() {
    let attempts = planned_attempts(wgpu::Backends::VULKAN, wgpu::Backends::DX12, None, false);
    let labels: Vec<&str> = attempts.iter().map(|attempt| attempt.label).collect();
    assert_eq!(labels, vec!["dx12", "software"]);
}
