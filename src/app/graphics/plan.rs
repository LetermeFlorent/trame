use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Attempt {
    pub(super) backends: wgpu::Backends,
    pub(super) software: bool,
    pub(super) label: &'static str,
}

impl Attempt {
    fn hardware(backends: wgpu::Backends, label: &'static str) -> Self {
        Self {
            backends,
            software: false,
            label,
        }
    }

    fn software(backends: wgpu::Backends) -> Self {
        Self {
            backends,
            software: true,
            label: "software",
        }
    }
}

pub(super) fn planned_attempts(
    preferred: wgpu::Backends,
    enabled: wgpu::Backends,
    env: Option<wgpu::Backends>,
    force_software: bool,
) -> Vec<Attempt> {
    if force_software {
        return vec![Attempt::software(enabled)];
    }
    let mut attempts = Vec::new();
    if let Some(env_backends) = env {
        attempts.push(Attempt::hardware(env_backends, backend_label(env_backends)));
    }
    if Some(preferred) != env && enabled.contains(preferred) {
        attempts.push(Attempt::hardware(preferred, backend_label(preferred)));
    }
    let candidates = [
        (wgpu::Backends::VULKAN, "vulkan"),
        (wgpu::Backends::METAL, "metal"),
        (wgpu::Backends::DX12, "dx12"),
        (wgpu::Backends::GL, "gl"),
    ];
    for (backend, label) in candidates {
        if enabled.contains(backend) && backend != preferred && Some(backend) != env {
            attempts.push(Attempt::hardware(backend, label));
        }
    }
    attempts.push(Attempt::software(enabled));
    attempts
}

pub(super) fn try_connect<T, E: Display>(
    attempts: &[Attempt],
    mut open: impl FnMut(Attempt) -> Result<(T, String), E>,
    log: &mut dyn FnMut(&str),
) -> Result<T, E> {
    for (index, &attempt) in attempts.iter().enumerate() {
        match open(attempt) {
            Ok((value, adapter_name)) => {
                if attempt.software {
                    log(&format!(
                        "using the software graphics adapter: {adapter_name}"
                    ));
                } else if index > 0 {
                    log(&format!(
                        "using the {} graphics adapter: {adapter_name}",
                        attempt.label
                    ));
                }
                return Ok(value);
            }
            Err(error) => {
                if let Some(next) = attempts.get(index + 1) {
                    log(&format!(
                        "the {} graphics backend failed ({error}), falling back to {}",
                        attempt.label, next.label
                    ));
                } else {
                    log(&format!(
                        "the {} graphics backend failed ({error})",
                        attempt.label
                    ));
                    return Err(error);
                }
            }
        }
    }
    unreachable!("planned_attempts always includes at least one attempt")
}

fn backend_label(backends: wgpu::Backends) -> &'static str {
    if backends == wgpu::Backends::VULKAN {
        "vulkan"
    } else if backends == wgpu::Backends::DX12 {
        "dx12"
    } else if backends == wgpu::Backends::METAL {
        "metal"
    } else if backends == wgpu::Backends::GL {
        "gl"
    } else {
        "configured"
    }
}
